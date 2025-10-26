use arrow::datatypes::{DataType, TimeUnit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::from_utf8;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

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
    index: bool,
    #[serde(default)]
    xsd: XsdType,
    #[serde(default)]
    references: Option<Reference>,
}

impl Into<arrow::datatypes::Field> for &Field {
    fn into(self) -> arrow::datatypes::Field {
        let data_type = Into::<arrow::datatypes::DataType>::into(&self.xsd);
        let mut field = arrow::datatypes::Field::new(&self.name, data_type, true);
        if self.index {
            field = field.with_metadata(HashMap::from([("index".to_string(), "true".to_string())]));
        }
        if let Some(Reference { table, column }) = &self.references {
            field = field.with_metadata(HashMap::from([
                ("references_table".to_string(), table.to_string()),
                ("references_column".to_string(), column.to_string()),
            ]));
        }
        field
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

// fn duckdb_schema<'a>(spec: Schema<'a>) -> String {
//     let columns = spec
//         .fields
//         .values()
//         .map(|field| to_duckdb_schema(field))
//         .collect::<Vec<_>>()
//         .join(",\n    ");
//     let primary = if let Some(primary) = spec.primary {
//         &format!(",\nprimary key (\"{}\")", primary)
//     } else {
//         ""
//     };
//     format!(
//         "create table if not exists \"{}\" (\n{}{}))",
//         spec.element, columns, primary
//     )
// }

#[derive(Debug)]
enum ParserState {
    StartRoot,
    StartElementOrEndRoot,
    StartAttrOrEndElement,
    AttrCdataOrEndAttr(String),
    Done,
}

fn parse<'a>(spec: &Schema, mut reader: Reader<&[u8]>) -> HashMap<String, Vec<String>> {
    let mut columns: HashMap<String, Vec<String>> = HashMap::new();
    let mut state = ParserState::StartRoot;
    let mut buf = Vec::new();
    loop {
        match (&state, reader.read_event_into(&mut buf)) {
            (state, Err(e)) => panic!(
                "Error in state {:?} at position {}: {:?}",
                state,
                reader.error_position(),
                e
            ),
            (ParserState::StartRoot, Ok(Event::Start(e))) => {
                if spec.root.as_bytes() == e.name().as_ref() {
                    state = ParserState::StartElementOrEndRoot;
                } else {
                    panic!(
                        "Expected root element {}, got {:?}",
                        spec.root,
                        e.name().as_ref()
                    )
                }
            }
            (ParserState::StartElementOrEndRoot, Ok(Event::Start(e))) => {
                let name = e.name();
                let name = from_utf8(name.as_ref()).unwrap();
                if spec.element != name {
                    panic!("Unknown element {}", name)
                }
                columns.values_mut().for_each(|v| v.push("".to_string()));
                state = ParserState::StartAttrOrEndElement
            }
            (ParserState::StartAttrOrEndElement, Ok(Event::Start(e))) => {
                let name = e.name();
                let name = from_utf8(name.as_ref()).unwrap();
                if !columns.contains_key(name) {
                    panic!("Unknown attribute {}", name)
                }
                state = ParserState::AttrCdataOrEndAttr(name.to_string())
            }
            (ParserState::AttrCdataOrEndAttr(element), Ok(Event::Text(e))) => {
                let content = e.xml_content().unwrap();
                columns
                    .get_mut(element)
                    .unwrap()
                    .last_mut()
                    .unwrap()
                    .push_str(content.as_ref());
            }
            (ParserState::AttrCdataOrEndAttr(element), Ok(Event::End(e))) => {
                let name = e.name();
                let name = from_utf8(name.as_ref()).unwrap();
                if name != element {
                    panic!("Expected closing of tag {}, got {}", element, name)
                }
                state = ParserState::StartAttrOrEndElement
            }
            (ParserState::StartAttrOrEndElement, Ok(Event::End(e))) => {
                let name = e.name();
                let name = from_utf8(name.as_ref()).unwrap();
                if spec.element != name {
                    panic!("Expected closing of tag {}, got {}", spec.element, name)
                }
                state = ParserState::StartElementOrEndRoot
            }
            (ParserState::StartElementOrEndRoot, Ok(Event::End(e))) => {
                let name = e.name();
                let name = from_utf8(name.as_ref()).unwrap();
                if spec.root != name {
                    panic!("Expected closing of tag {}, got {}", spec.element, name)
                }
                state = ParserState::Done
            }
            (ParserState::Done, Ok(Event::Eof)) => break,
            (
                _,
                Ok(Event::Comment(_))
                | Ok(Event::PI(_))
                | Ok(Event::DocType(_))
                | Ok(Event::Empty(_)),
            ) => {
                // Ignore
            }
            (state, event) => {
                panic!("Unexpected event {:?} in state {:?}", state, event)
            }
        }
        buf.clear();
    }
    columns
}

fn main() {
    let path = std::path::PathBuf::from(std::env::args().nth(1).unwrap());
    let file = std::fs::File::open(path).unwrap();
    let schema: Schema = serde_json::from_reader(file).unwrap();
    println!("{:?}", schema);
}
