use anyhow::{Result, anyhow};
use arrow::array::{RecordBatch, StringBuilder};
use std::sync::Arc;
use xml::{
    name::OwnedName,
    reader::{EventReader, ParserConfig, XmlEvent},
};

use crate::schema::Schema;

/// Represents the different states of the XML parser state machine
#[derive(Debug)]
enum ParserState {
    /// Initial state - expecting document start
    StartDocument,
    /// Expecting the root element to start
    StartRoot,
    /// Expecting either a new element or the end of root
    StartElementOrEndRoot,
    /// Expecting either an attribute start or element end
    StartAttrOrEndElement,
    /// Reading attribute content or expecting attribute end
    AttrCdataOrEndAttr(usize),
    /// Parsing complete
    Done,
}

/// XML parser for converting XML data to Arrow RecordBatch according to a schema
pub struct XmlParser;

impl XmlParser {
    /// Parse XML data from a reader into an Arrow RecordBatch
    ///
    /// # Arguments
    /// * `schema` - The schema definition for parsing
    /// * `reader` - XML event reader
    ///
    /// # Returns
    /// * `Result<RecordBatch>` - The parsed data as an Arrow RecordBatch
    pub fn parse<R>(schema: &Schema, reader: EventReader<R>) -> Result<RecordBatch>
    where
        R: std::io::BufRead,
    {
        let fields = arrow::datatypes::Schema::from(schema).fields().clone();

        // Initialize string builders for each field
        // TODO: Magic numbers should be constants or configurable
        let mut builders = fields
            .iter()
            .map(|_| StringBuilder::with_capacity(100_000, 100_000 * 64))
            .collect::<Vec<StringBuilder>>();

        let mut state = ParserState::StartDocument;

        // Track current values being accumulated for each field
        let mut current_values: Vec<String> =
            fields.iter().map(|_| String::with_capacity(1024)).collect();

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
                    let idx = fields
                        .find(&local_name)
                        .ok_or_else(|| anyhow!("Unknown attribute {}", local_name))?
                        .0;
                    state = ParserState::AttrCdataOrEndAttr(idx)
                }
                (ParserState::AttrCdataOrEndAttr(idx), Ok(XmlEvent::Characters(content))) => {
                    // TODO: String concatenation can be expensive - consider using a more efficient buffer
                    // Accumulate content for this element
                    if let Some(existing_content) = current_values.get_mut(*idx) {
                        existing_content.push_str(&content);
                    } else {
                        return Err(anyhow!("Element {} not found in schema", idx));
                    }
                }
                (ParserState::AttrCdataOrEndAttr(_idx), Ok(XmlEvent::EndElement { .. })) => {
                    // TODO(leo): Bring this check back
                    // let OwnedName { local_name, .. } = name;
                    // if &local_name != element {
                    //     return Err(anyhow!(
                    //         "Expected closing of tag {}, got {}",
                    //         element,
                    //         local_name
                    //     ));
                    // }
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
                    Self::flush_current_values(&fields, &mut current_values, &mut builders)?;
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

        Self::build_record_batch(schema, builders)
    }

    /// Create an XML parser with configuration
    pub fn create_reader<R>(reader: R) -> EventReader<R>
    where
        R: std::io::BufRead,
    {
        ParserConfig::new()
            .trim_whitespace(true)
            .ignore_comments(true)
            .create_reader(reader)
    }

    /// Flush current accumulated values to the string builders
    fn flush_current_values(
        fields: &arrow::datatypes::Fields,
        current_values: &mut Vec<String>,
        builders: &mut Vec<StringBuilder>,
    ) -> Result<()> {
        for (idx, field) in fields.iter().enumerate() {
            let field_name = field.name();
            let value_builder = current_values
                .get_mut(idx)
                .ok_or_else(|| anyhow!("Field {} not found in current values", field_name))?;

            if value_builder.is_empty() {
                builders
                    .get_mut(idx)
                    .ok_or_else(|| anyhow!("Builder for field {} not found", field_name))?
                    .append_null();
            } else {
                let drained_value: String = value_builder.drain(..).collect();
                builders
                    .get_mut(idx)
                    .ok_or_else(|| anyhow!("Builder for field {} not found", field_name))?
                    .append_value(drained_value);
            }
        }
        Ok(())
    }

    /// Convert string builders to a RecordBatch with proper type casting
    fn build_record_batch(schema: &Schema, builders: Vec<StringBuilder>) -> Result<RecordBatch> {
        let arrow_schema = Arc::new(arrow::datatypes::Schema::from(schema));

        let mut columns = Vec::with_capacity(schema.fields.len());
        builders
            .into_iter()
            .enumerate()
            .try_for_each(|(i, mut builder)| {
                let field = arrow_schema
                    .fields()
                    .get(i)
                    .ok_or_else(|| anyhow!("Field {} not found", i))?;
                let string_array = builder.finish();
                let casted_array = arrow_cast::cast(&string_array, &field.data_type())
                    .map_err(|e| anyhow!("Failed to cast field {}: {}", field.name(), e))?;
                columns.push(casted_array);
                Result::<()>::Ok(())
            })?;

        let record_batch = RecordBatch::try_new(arrow_schema, columns)
            .map_err(|e| anyhow!("Failed to create RecordBatch: {}", e))?;

        Ok(record_batch)
    }
}
