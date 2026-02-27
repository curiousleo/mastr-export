use anyhow::{Result, anyhow};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::io::BufRead;

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

/// XML parser for converting XML data to columnar string data according to a schema
pub struct XmlParser;

impl XmlParser {
    /// Parse XML data from a reader into columns of nullable strings.
    pub fn parse<R>(schema: &Schema, reader: R) -> Result<Vec<Vec<Option<String>>>>
    where
        R: BufRead,
    {
        let num_fields = schema.fields.len();

        let field_indices: HashMap<&str, usize> = schema
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.as_str(), i))
            .collect();

        let mut columns: Vec<Vec<Option<String>>> =
            (0..num_fields).map(|_| Vec::with_capacity(100_000)).collect();

        let mut current_values: Vec<String> =
            (0..num_fields).map(|_| String::with_capacity(1024)).collect();

        let mut state = ParserState::StartDocument;

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
                            let idx = field_indices
                                .get(name)
                                .copied()
                                .ok_or_else(|| anyhow!("Unknown attribute {}", name))?;
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

                            Self::flush_current_values(&mut current_values, &mut columns);
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
                    if let ParserState::AttrCdataOrEndAttr(idx) = &state {
                        let content = e
                            .unescape()
                            .map_err(|e| anyhow!("Failed to unescape text content: {}", e))?;
                        current_values[*idx].push_str(&content);
                    }
                }
                Ok(Event::CData(ref e)) => {
                    if let ParserState::AttrCdataOrEndAttr(idx) = &state {
                        let content = std::str::from_utf8(e.as_ref())
                            .map_err(|e| anyhow!("Invalid UTF-8 in CDATA: {}", e))?;
                        current_values[*idx].push_str(content);
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
                | Ok(Event::DocType(_)) => {}
                Ok(Event::Empty(_)) => {}
                Err(e) => {
                    return Err(anyhow!("XML parsing error in state {:?}: {}", state, e));
                }
            }

            buf.clear();
        }

        Ok(columns)
    }

    fn flush_current_values(
        current_values: &mut [String],
        columns: &mut [Vec<Option<String>>],
    ) {
        for (idx, value) in current_values.iter_mut().enumerate() {
            if value.is_empty() {
                columns[idx].push(None);
            } else {
                columns[idx].push(Some(value.drain(..).collect()));
            }
        }
    }
}
