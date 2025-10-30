// TODO: Move schema types and XML parser to separate modules for better organization
use anyhow::{Result, anyhow};
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
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

impl From<XsdType> for DataType {
    fn from(xsd_type: XsdType) -> Self {
        match xsd_type {
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

impl From<&Field> for arrow::datatypes::Field {
    fn from(field: &Field) -> Self {
        let data_type = DataType::from(field.xsd);
        arrow::datatypes::Field::new(&field.name, data_type, true)
    }
}

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

// TODO: Consider using string constants for metadata keys to avoid typos
// TODO: This should consume self, not take &self for better performance
impl From<&Schema> for arrow::datatypes::Schema {
    fn from(schema: &Schema) -> Self {
        let fields = schema
            .fields
            .iter()
            .map(|field| arrow::datatypes::Field::from(field))
            .collect::<Vec<_>>();
        let fields = arrow::datatypes::Fields::from(fields);
        let mut metadata = HashMap::from([
            ("root".to_string(), schema.root.to_string()),
            ("element".to_string(), schema.element.to_string()),
        ]);
        if schema.without_rowid {
            metadata.insert("without_rowid".to_string(), "true".to_string());
        }
        if let Some(primary) = &schema.primary {
            metadata.insert("primary".to_string(), primary.to_string());
        }
        arrow::datatypes::Schema::new(fields).with_metadata(metadata)
    }
}

// TODO: Consider moving parser state and logic to a separate Parser struct
// TODO: Add documentation comments for each state
#[derive(Debug)]
enum ParserState {
    StartDocument,
    StartRoot,
    StartElementOrEndRoot,
    StartAttrOrEndElement,
    AttrCdataOrEndAttr(String),
    Done,
}

// TODO: This function is too large - consider breaking into smaller functions
fn parse<R>(schema: &Schema, reader: EventReader<R>) -> Result<RecordBatch>
where
    R: std::io::BufRead,
{
    let fields = arrow::datatypes::Schema::from(schema).fields().clone();
    // TODO: Magic numbers should be constants or configurable
    // TODO: Consider using Vec instead of HashMap for better performance with indexed access
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
    // TODO: Consider using Vec<String> indexed by field position for better performance
    let mut current_values: HashMap<String, String> = fields
        .iter()
        .map(|field| (field.name().to_string(), String::new()))
        .collect();
    for event in reader {
        match (&mut state, event) {
            (ParserState::StartDocument, Ok(XmlEvent::StartDocument { .. })) => {
                state = ParserState::StartRoot;
            }
            (state, Err(e)) => {
                return Err(anyhow!("XML parsing error in state {:?}: {}", state, e));
            }
            (ParserState::StartRoot, Ok(XmlEvent::Characters(_))) => {
                // Leading text, ignore
            }
            (ParserState::StartRoot, Ok(XmlEvent::StartElement { name, .. })) => {
                let OwnedName { local_name, .. } = name;
                if schema.root == local_name {
                    state = ParserState::StartElementOrEndRoot;
                } else {
                    return Err(anyhow!(
                        "Expected root element {}, got {}",
                        schema.root,
                        local_name
                    ));
                }
            }
            (ParserState::StartElementOrEndRoot, Ok(XmlEvent::StartElement { name, .. })) => {
                let OwnedName { local_name, .. } = name;
                if schema.element != local_name {
                    return Err(anyhow!("Unknown element {}", local_name));
                }
                state = ParserState::StartAttrOrEndElement
            }
            (ParserState::StartAttrOrEndElement, Ok(XmlEvent::StartElement { name, .. })) => {
                let OwnedName { local_name, .. } = name;
                state = ParserState::AttrCdataOrEndAttr(local_name)
            }
            (ParserState::AttrCdataOrEndAttr(element), Ok(XmlEvent::Characters(content))) => {
                // TODO: String concatenation can be expensive - consider using a more efficient buffer
                // Accumulate content for this element
                if let Some(existing_content) = current_values.get_mut(element) {
                    existing_content.push_str(&content);
                } else {
                    return Err(anyhow!("Element {} not found in schema", element));
                }
            }
            (ParserState::AttrCdataOrEndAttr(element), Ok(XmlEvent::EndElement { name })) => {
                let OwnedName { local_name, .. } = name;
                if &local_name != element {
                    return Err(anyhow!(
                        "Expected closing of tag {}, got {}",
                        element,
                        local_name
                    ));
                }
                state = ParserState::StartAttrOrEndElement
            }
            (ParserState::StartAttrOrEndElement, Ok(XmlEvent::EndElement { name })) => {
                let OwnedName { local_name, .. } = name;
                if schema.element != local_name {
                    return Err(anyhow!(
                        "Expected closing of tag {}, got {}",
                        schema.element,
                        local_name
                    ));
                }
                // Append accumulated values to builders
                // TODO: Consider pre-computing field indices for faster access
                for field in &fields {
                    let field_name = field.name();
                    let value_builder = current_values.get_mut(field_name).ok_or_else(|| {
                        anyhow!("Field {} not found in current values", field_name)
                    })?;
                    if value_builder.is_empty() {
                        builders
                            .get_mut(field_name)
                            .ok_or_else(|| anyhow!("Builder for field {} not found", field_name))?
                            .append_null();
                    } else {
                        let drained_value: String = value_builder.drain(..).collect();
                        builders
                            .get_mut(field_name)
                            .ok_or_else(|| anyhow!("Builder for field {} not found", field_name))?
                            .append_value(drained_value);
                    }
                }
                state = ParserState::StartElementOrEndRoot
            }
            (ParserState::StartElementOrEndRoot, Ok(XmlEvent::EndElement { name })) => {
                let OwnedName { local_name, .. } = name;
                if schema.root != local_name {
                    return Err(anyhow!(
                        "Expected closing of root tag {}, got {}",
                        schema.root,
                        local_name
                    ));
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
                return Err(anyhow!("Unexpected event {:?} in state {:?}", event, state));
            }
        }
    }

    // Convert builders to arrays
    let schema = Arc::new(arrow::datatypes::Schema::from(schema));
    let columns = schema
        .fields()
        .iter()
        .map(|field| -> Result<_> {
            let builder = builders
                .get_mut(field.name())
                .ok_or_else(|| anyhow!("Builder for field {} not found", field.name()))?;
            let string_array = builder.finish();
            let casted_array = arrow_cast::cast(&string_array, &field.data_type())
                .map_err(|e| anyhow!("Failed to cast field {}: {}", field.name(), e))?;
            Ok(casted_array)
        })
        .collect::<Result<Vec<_>>>()?;
    let record_batch = RecordBatch::try_new(schema, columns)
        .map_err(|e| anyhow!("Failed to create RecordBatch: {}", e))?;
    Ok(record_batch)
}

// TODO: Use clap or similar for proper CLI argument parsing
fn main() -> Result<()> {
    let schema_path = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or_else(|| anyhow!("Missing schema file argument"))?,
    );
    let schema_file = std::fs::File::open(&schema_path)
        .map_err(|e| anyhow!("Failed to open schema file {:?}: {}", schema_path, e))?;
    let schema: Schema = serde_json::from_reader(schema_file)
        .map_err(|e| anyhow!("Failed to parse schema JSON: {}", e))?;

    println!("{:?}", arrow::datatypes::Schema::from(&schema));

    let zip_path = std::path::PathBuf::from(
        std::env::args()
            .nth(2)
            .ok_or_else(|| anyhow!("Missing ZIP file argument"))?,
    );
    let zip_file = std::fs::File::open(&zip_path)
        .map_err(|e| anyhow!("Failed to open ZIP file {:?}: {}", zip_path, e))?;

    let xml_name = std::env::args()
        .nth(3)
        .ok_or_else(|| anyhow!("Missing XML file name argument"))?;

    let mut archive =
        zip::ZipArchive::new(zip_file).map_err(|e| anyhow!("Failed to read ZIP archive: {}", e))?;
    let file = archive
        .by_name(&xml_name)
        .map_err(|e| anyhow!("Failed to find XML file '{}' in archive: {}", xml_name, e))?;
    let buf_reader = std::io::BufReader::new(file);
    let reader = ParserConfig::new()
        .trim_whitespace(true)
        .ignore_comments(true)
        .create_reader(buf_reader);
    let record_batch = parse(&schema, reader)?;
    println!("Number of columns: {}", record_batch.num_columns());
    println!("Number of rows: {}", record_batch.num_rows());
    Ok(())
}
