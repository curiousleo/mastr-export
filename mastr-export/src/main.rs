use arrow::{
    array::{RecordBatch, StringBuilder},
    datatypes::{DataType, TimeUnit},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use xml::{
    name::OwnedName,
    reader::{EventReader, ParserConfig, XmlEvent},
};

#[derive(Debug, Serialize, Deserialize)]
struct Reference {
    table: String,
    column: String,
}

// fn to_sqlite_schema(reference: &Reference) -> String {
//     format!(
//         "references \"{}\"(\"{}\")",
//         reference.table, reference.column
//     )
// }

#[derive(Debug, Serialize, Deserialize)]
enum XsdType {
    #[serde(rename = "date")]
    Date,
    #[serde(rename = "dateTime")]
    DateTime,
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "double")]
    Double,
    #[serde(rename = "byte")]
    Byte,
    #[serde(rename = "short")]
    Short,
    #[serde(rename = "int")]
    Int,
    #[serde(rename = "nonNegativeInteger")]
    NonNegativeInteger,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "string")]
    String,
}

impl Default for XsdType {
    fn default() -> Self {
        XsdType::String
    }
}

impl Into<DataType> for &XsdType {
    fn into(self) -> DataType {
        match self {
            XsdType::Date => DataType::Date32,
            XsdType::DateTime => DataType::Timestamp(TimeUnit::Second, None),
            XsdType::Float => DataType::Float32,
            XsdType::Double => DataType::Float64,
            XsdType::Byte => DataType::Int8,
            XsdType::Short => DataType::Int16,
            XsdType::Int => DataType::Int32,
            XsdType::NonNegativeInteger => DataType::UInt64,
            XsdType::Boolean => DataType::Boolean,
            XsdType::String => DataType::Utf8,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Field {
    name: String,
    #[serde(default)]
    xsd: XsdType,
}

impl Into<arrow::datatypes::Field> for &Field {
    fn into(self) -> arrow::datatypes::Field {
        let data_type = Into::<arrow::datatypes::DataType>::into(&self.xsd);
        arrow::datatypes::Field::new(&self.name, data_type, true)
    }
}

// fn to_duckdb_schema(field: &Field) -> String {
//     let duckdb_type = duckdb_type_to_string(to_duckdb_type(&field.xsd));
//     format!("{} {}", field.name, duckdb_type)
// }

#[derive(Debug, Serialize, Deserialize)]
struct Schema {
    root: String,
    element: String,
    #[serde(default)]
    without_rowid: bool,
    #[serde(default)]
    primary: Option<String>,
    fields: Vec<Field>,
}

impl Into<arrow::datatypes::Schema> for &Schema {
    fn into(self) -> arrow::datatypes::Schema {
        let fields = self
            .fields
            .iter()
            .map(|field| Into::<arrow::datatypes::Field>::into(field))
            .collect::<Vec<_>>();
        let fields = arrow::datatypes::Fields::from(fields);
        let mut metadata = HashMap::from([
            ("root".to_string(), self.root.to_string()),
            ("element".to_string(), self.element.to_string()),
        ]);
        if self.without_rowid {
            metadata.insert("without_rowid".to_string(), "true".to_string());
        }
        if let Some(primary) = &self.primary {
            metadata.insert("primary".to_string(), primary.to_string());
        }
        arrow::datatypes::Schema::new(fields).with_metadata(metadata)
    }
}

#[derive(Debug)]
enum ParserState {
    StartDocument,
    StartRoot,
    StartElementOrEndRoot,
    StartAttrOrEndElement,
    AttrCdataOrEndAttr(String),
    Done,
}

fn parse<R>(schema: &Schema, reader: EventReader<R>) -> RecordBatch
where
    R: std::io::BufRead,
{
    let fields = Into::<arrow::datatypes::Schema>::into(schema)
        .fields()
        .clone();
    let mut builders = fields
        .iter()
        .map(|field| {
            (
                field.name().to_string(),
                StringBuilder::with_capacity(100_000, 100_000 * 32),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut state = ParserState::StartDocument;
    let mut current_values: HashMap<String, String> = fields
        .iter()
        .map(|field| (field.name().to_string(), String::new()))
        .collect();
    for event in reader {
        match (&mut state, event) {
            (ParserState::StartDocument, Ok(XmlEvent::StartDocument { .. })) => {
                state = ParserState::StartRoot;
            }
            (state, Err(e)) => panic!("Error in state {:?}: {:?}", state, e),
            (ParserState::StartRoot, Ok(XmlEvent::Characters(_))) => {
                // Leading text, ignore
            }
            (ParserState::StartRoot, Ok(XmlEvent::StartElement { name, .. })) => {
                let OwnedName { local_name, .. } = name;
                if schema.root == local_name {
                    state = ParserState::StartElementOrEndRoot;
                } else {
                    panic!(
                        "Expected root element {}, got {:?}",
                        schema.root, local_name
                    )
                }
            }
            (ParserState::StartElementOrEndRoot, Ok(XmlEvent::StartElement { name, .. })) => {
                let OwnedName { local_name, .. } = name;
                if schema.element != local_name {
                    panic!("Unknown element {}", local_name)
                }
                state = ParserState::StartAttrOrEndElement
            }
            (ParserState::StartAttrOrEndElement, Ok(XmlEvent::StartElement { name, .. })) => {
                let OwnedName { local_name, .. } = name;
                state = ParserState::AttrCdataOrEndAttr(local_name)
            }
            (ParserState::AttrCdataOrEndAttr(element), Ok(XmlEvent::Characters(content))) => {
                // Accumulate content for this element
                if let Some(existing_content) = current_values.get_mut(element) {
                    existing_content.push_str(&content);
                } else {
                    panic!("Element {} not found in builders", element);
                }
            }
            (ParserState::AttrCdataOrEndAttr(element), Ok(XmlEvent::EndElement { name })) => {
                let OwnedName { local_name, .. } = name;
                if &local_name != element {
                    panic!("Expected closing of tag {}, got {}", element, local_name)
                }
                state = ParserState::StartAttrOrEndElement
            }
            (ParserState::StartAttrOrEndElement, Ok(XmlEvent::EndElement { name })) => {
                let OwnedName { local_name, .. } = name;
                if schema.element != local_name {
                    panic!(
                        "Expected closing of tag {}, got {}",
                        schema.element, local_name
                    )
                }
                // Append accumulated values to builders
                for field in &fields {
                    let field_name = field.name();
                    let value_builder = current_values.get_mut(field_name).unwrap();
                    if value_builder.is_empty() {
                        builders.get_mut(field_name).unwrap().append_null();
                    } else {
                        let drained_value: String = value_builder.drain(..).collect();
                        builders
                            .get_mut(field_name)
                            .unwrap()
                            .append_value(drained_value);
                    }
                }
                state = ParserState::StartElementOrEndRoot
            }
            (ParserState::StartElementOrEndRoot, Ok(XmlEvent::EndElement { name })) => {
                let OwnedName { local_name, .. } = name;
                if schema.root != local_name {
                    panic!(
                        "Expected closing of tag {}, got {}",
                        schema.element, local_name
                    )
                }
                state = ParserState::Done
            }
            (ParserState::Done, Ok(XmlEvent::EndDocument)) => {
                // Done
            }
            (
                _,
                Ok(XmlEvent::Comment(_))
                | Ok(XmlEvent::CData(_))
                | Ok(XmlEvent::Doctype { .. })
                | Ok(XmlEvent::ProcessingInstruction { .. }),
            ) => {
                // Ignore
            }
            (state, event) => {
                panic!("Unexpected event {:?} in state {:?}", state, event)
            }
        }
    }

    // Convert builders to arrays
    let schema = Arc::new(Into::<arrow::datatypes::Schema>::into(schema));
    let columns = schema
        .fields()
        .iter()
        .map(|field| {
            let builder = builders.get_mut(field.name()).unwrap();
            let string_array = builder.finish();
            arrow_cast::cast(&string_array, &field.data_type()).unwrap()
        })
        .collect::<Vec<_>>();
    RecordBatch::try_new(schema, columns).unwrap()
}

fn main() {
    let schema_path = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    let schema_file = std::fs::File::open(schema_path).unwrap();
    let schema: Schema = serde_json::from_reader(schema_file).unwrap();

    println!("{:?}", Into::<arrow::datatypes::Schema>::into(&schema));

    let zip_path = std::path::PathBuf::from(std::env::args().nth(2).unwrap());
    let zip_file = std::fs::File::open(zip_path).unwrap();

    let xml_name = std::env::args().nth(3).unwrap();

    let mut archive = zip::ZipArchive::new(zip_file).unwrap();
    let file = archive.by_name(&xml_name).unwrap();
    let buf_reader = std::io::BufReader::new(file);
    let reader = ParserConfig::new()
        .trim_whitespace(true)
        .ignore_comments(true)
        .create_reader(buf_reader);
    let record_batch = parse(&schema, reader);
    println!("Number of columns: {}", record_batch.num_columns());
    println!("Number of rows: {}", record_batch.num_rows());
}
