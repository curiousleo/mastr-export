use arrow::{
    array::{
        ArrayBuilder, BooleanBuilder, Date32Builder, Float32Builder, Float64Builder, Int8Builder,
        Int16Builder, Int32Builder, RecordBatch, StringBuilder, TimestampSecondBuilder,
        UInt64Builder,
    },
    datatypes::{DataType, TimeUnit},
};
use chrono::{NaiveDate, NaiveDateTime};
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

fn append_value_to_builder(builder: &mut Box<dyn ArrayBuilder>, value: &str, data_type: &DataType) {
    match data_type {
        DataType::Date32 => {
            let builder = builder
                .as_any_mut()
                .downcast_mut::<Date32Builder>()
                .unwrap();
            if value.is_empty() {
                builder.append_null();
            } else {
                // Try to parse date in common formats
                let parsed_date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
                    .or_else(|_| NaiveDate::parse_from_str(value, "%d.%m.%Y"))
                    .or_else(|_| NaiveDate::parse_from_str(value, "%m/%d/%Y"));

                match parsed_date {
                    Ok(date) => {
                        // Convert to days since epoch (1970-01-01)
                        let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
                        let days = date.signed_duration_since(epoch).num_days() as i32;
                        builder.append_value(days);
                    }
                    Err(_) => builder.append_null(),
                }
            }
        }
        DataType::Timestamp(TimeUnit::Second, None) => {
            let builder = builder
                .as_any_mut()
                .downcast_mut::<TimestampSecondBuilder>()
                .unwrap();
            if value.is_empty() {
                builder.append_null();
            } else {
                // Try to parse timestamp in common formats
                let parsed_datetime = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
                    .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S"))
                    .or_else(|_| NaiveDateTime::parse_from_str(value, "%d.%m.%Y %H:%M:%S"));

                match parsed_datetime {
                    Ok(datetime) => {
                        let timestamp = datetime.and_utc().timestamp();
                        builder.append_value(timestamp);
                    }
                    Err(_) => builder.append_null(),
                }
            }
        }
        DataType::Float32 => {
            let builder = builder
                .as_any_mut()
                .downcast_mut::<Float32Builder>()
                .unwrap();
            if value.is_empty() {
                builder.append_null();
            } else {
                match value.parse::<f32>() {
                    Ok(v) => builder.append_value(v),
                    Err(_) => builder.append_null(),
                }
            }
        }
        DataType::Float64 => {
            let builder = builder
                .as_any_mut()
                .downcast_mut::<Float64Builder>()
                .unwrap();
            if value.is_empty() {
                builder.append_null();
            } else {
                match value.parse::<f64>() {
                    Ok(v) => builder.append_value(v),
                    Err(_) => builder.append_null(),
                }
            }
        }
        DataType::Int8 => {
            let builder = builder.as_any_mut().downcast_mut::<Int8Builder>().unwrap();
            if value.is_empty() {
                builder.append_null();
            } else {
                match value.parse::<i8>() {
                    Ok(v) => builder.append_value(v),
                    Err(_) => builder.append_null(),
                }
            }
        }
        DataType::Int16 => {
            let builder = builder.as_any_mut().downcast_mut::<Int16Builder>().unwrap();
            if value.is_empty() {
                builder.append_null();
            } else {
                match value.parse::<i16>() {
                    Ok(v) => builder.append_value(v),
                    Err(_) => builder.append_null(),
                }
            }
        }
        DataType::Int32 => {
            let builder = builder.as_any_mut().downcast_mut::<Int32Builder>().unwrap();
            if value.is_empty() {
                builder.append_null();
            } else {
                match value.parse::<i32>() {
                    Ok(v) => builder.append_value(v),
                    Err(_) => builder.append_null(),
                }
            }
        }
        DataType::UInt64 => {
            let builder = builder
                .as_any_mut()
                .downcast_mut::<UInt64Builder>()
                .unwrap();
            if value.is_empty() {
                builder.append_null();
            } else {
                match value.parse::<u64>() {
                    Ok(v) => builder.append_value(v),
                    Err(_) => builder.append_null(),
                }
            }
        }
        DataType::Boolean => {
            let builder = builder
                .as_any_mut()
                .downcast_mut::<BooleanBuilder>()
                .unwrap();
            if value.is_empty() {
                builder.append_null();
            } else {
                match value.to_lowercase().as_str() {
                    "true" | "1" | "yes" => builder.append_value(true),
                    "false" | "0" | "no" => builder.append_value(false),
                    _ => builder.append_null(),
                }
            }
        }
        DataType::Utf8 => {
            let builder = builder
                .as_any_mut()
                .downcast_mut::<StringBuilder>()
                .unwrap();
            if value.is_empty() {
                builder.append_null();
            } else {
                builder.append_value(value);
            }
        }
        _ => unimplemented!(),
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
    StartDocument,
    StartRoot,
    StartElementOrEndRoot,
    StartAttrOrEndElement,
    AttrCdataOrEndAttr(String),
    Done,
}

fn array_builder_from_data_type(data_type: &DataType) -> Box<dyn ArrayBuilder> {
    match data_type {
        DataType::Date32 => Box::new(Date32Builder::new()),
        DataType::Timestamp(TimeUnit::Second, None) => Box::new(TimestampSecondBuilder::new()),
        DataType::Float32 => Box::new(Float32Builder::new()),
        DataType::Float64 => Box::new(Float64Builder::new()),
        DataType::Int8 => Box::new(Int8Builder::new()),
        DataType::Int16 => Box::new(Int16Builder::new()),
        DataType::Int32 => Box::new(Int32Builder::new()),
        DataType::UInt64 => Box::new(UInt64Builder::new()),
        DataType::Boolean => Box::new(BooleanBuilder::new()),
        DataType::Utf8 => Box::new(StringBuilder::new()),
        _ => unimplemented!(),
    }
}

fn parse<R>(schema: &Schema, reader: EventReader<R>) -> RecordBatch
where
    R: std::io::BufRead,
{
    // TODO(leo): Accumulate into `StringArray`s and use
    // `arrow_cast::cast::cast` to convert them to the appropriate data types.
    let fields = Into::<arrow::datatypes::Schema>::into(schema)
        .fields()
        .clone();
    let mut builders = fields
        .iter()
        .map(|field| {
            (
                field.name().to_string(),
                array_builder_from_data_type(field.data_type()),
            )
        })
        .collect::<HashMap<String, Box<dyn ArrayBuilder>>>();

    // Create a mapping from field name to data type for easy lookup
    let field_types = fields
        .iter()
        .map(|field| (field.name().to_string(), field.data_type().clone()))
        .collect::<HashMap<String, DataType>>();
    let mut state = ParserState::StartDocument;
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
                if let (Some(builder), Some(data_type)) =
                    (builders.get_mut(element), field_types.get(element))
                {
                    append_value_to_builder(builder, content.as_ref(), data_type);
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
    let columns = builders
        .into_values()
        .map(|mut builder| (builder.finish()))
        .collect::<Vec<_>>();
    RecordBatch::try_new(
        Arc::new(Into::<arrow::datatypes::Schema>::into(schema)),
        columns,
    )
    .unwrap()
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
