use anyhow::{Result, anyhow};
use arrow::array::{RecordBatch, StringBuilder};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::BufRead;
use std::sync::Arc;

use crate::schema::Schema;

/// Represents the different states of the XML parser state machine
#[derive(Debug)]
enum ParserState {
    /// Initial state - expecting document start
    StartDocument,
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
    /// * `reader` - Buffered reader containing XML data
    ///
    /// # Returns
    /// * `Result<RecordBatch>` - The parsed data as an Arrow RecordBatch
    pub fn parse<R>(schema: &Schema, reader: R) -> Result<RecordBatch>
    where
        R: BufRead,
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

        let mut xml_reader = Reader::from_reader(reader);
        xml_reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = std::str::from_utf8(e.name().into_inner())
                        .map_err(|e| anyhow!("Invalid UTF-8 in element name: {}", e))?;

                    match &mut state {
                        ParserState::StartDocument => {
                            if schema.root == name {
                                state = ParserState::StartElementOrEndRoot;
                            } else {
                                return Err(anyhow!(
                                    "Expected root element {}, got {}",
                                    schema.root,
                                    name
                                ));
                            }
                        }
                        ParserState::StartElementOrEndRoot => {
                            if schema.element != name {
                                return Err(anyhow!("Unknown element {}", name));
                            }
                            state = ParserState::StartAttrOrEndElement;
                        }
                        ParserState::StartAttrOrEndElement => {
                            let idx = fields
                                .find(name)
                                .ok_or_else(|| anyhow!("Unknown attribute {}", name))?
                                .0;
                            state = ParserState::AttrCdataOrEndAttr(idx);
                        }
                        _ => {
                            return Err(anyhow!(
                                "Unexpected start element {} in state {:?}",
                                name,
                                state
                            ));
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = std::str::from_utf8(e.name().into_inner())
                        .map_err(|e| anyhow!("Invalid UTF-8 in element name: {}", e))?;

                    match &mut state {
                        ParserState::AttrCdataOrEndAttr(_idx) => {
                            // TODO(leo): Bring this check back
                            // if name != element {
                            //     return Err(anyhow!(
                            //         "Expected closing of tag {}, got {}",
                            //         element,
                            //         name
                            //     ));
                            // }
                            state = ParserState::StartAttrOrEndElement;
                        }
                        ParserState::StartAttrOrEndElement => {
                            if schema.element != name {
                                return Err(anyhow!(
                                    "Expected closing of tag {}, got {}",
                                    schema.element,
                                    name
                                ));
                            }

                            // Append accumulated values to builders
                            Self::flush_current_values(
                                &fields,
                                &mut current_values,
                                &mut builders,
                            )?;
                            state = ParserState::StartElementOrEndRoot;
                        }
                        ParserState::StartElementOrEndRoot => {
                            if schema.root != name {
                                return Err(anyhow!(
                                    "Expected closing of root tag {}, got {}",
                                    schema.root,
                                    name
                                ));
                            }
                            state = ParserState::Done;
                        }
                        _ => {
                            return Err(anyhow!(
                                "Unexpected end element {} in state {:?}",
                                name,
                                state
                            ));
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    match &state {
                        ParserState::AttrCdataOrEndAttr(idx) => {
                            let content = e
                                .unescape()
                                .map_err(|e| anyhow!("Failed to unescape text content: {}", e))?;

                            // TODO: String concatenation can be expensive - consider using a more efficient buffer
                            // Accumulate content for this element
                            if let Some(existing_content) = current_values.get_mut(*idx) {
                                existing_content.push_str(&content);
                            } else {
                                return Err(anyhow!("Element {} not found in schema", idx));
                            }
                        }
                        _ => {
                            // Ignore text in other states
                        }
                    }
                }
                Ok(Event::CData(ref e)) => {
                    match &state {
                        ParserState::AttrCdataOrEndAttr(idx) => {
                            let content = std::str::from_utf8(e.as_ref())
                                .map_err(|e| anyhow!("Invalid UTF-8 in CDATA: {}", e))?;

                            // Accumulate CDATA content for this element
                            if let Some(existing_content) = current_values.get_mut(*idx) {
                                existing_content.push_str(content);
                            } else {
                                return Err(anyhow!("Element {} not found in schema", idx));
                            }
                        }
                        _ => {
                            // Ignore CDATA in other states
                        }
                    }
                }
                Ok(Event::Eof) => {
                    if !matches!(state, ParserState::Done) {
                        return Err(anyhow!("Unexpected end of document in state {:?}", state));
                    }
                    break;
                }
                Ok(Event::Comment(_))
                | Ok(Event::Decl(_))
                | Ok(Event::PI(_))
                | Ok(Event::DocType(_)) => {
                    // Ignore these events
                }
                Ok(Event::Empty(_)) => {
                    // Handle self-closing tags if needed
                    // For now, treat as ignored
                }
                Err(e) => {
                    return Err(anyhow!("XML parsing error in state {:?}: {}", state, e));
                }
            }

            buf.clear();
        }

        Self::build_record_batch(schema, builders)
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
