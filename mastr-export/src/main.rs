use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::from_utf8;

use quick_xml::events::Event;
use quick_xml::reader::Reader;

#[derive(Serialize, Deserialize)]
struct Reference<'a> {
    table: &'a str,
    column: &'a str,
}

fn to_sqlite_schema(reference: &Reference) -> String {
    format!(
        "references \"{}\"(\"{}\")",
        reference.table, reference.column
    )
}

#[derive(Serialize, Deserialize)]
enum XsdType {
    Date,
    DateTime,
    Float,
    Double,
    Decimal,
    Byte,
    Short,
    Int,
    NonNegativeInteger,
    Boolean,
    String,
}

enum DuckDBType {
    Date,
    Timestamp,
    Float,
    Double,
    Tinyint,
    Smallint,
    Integer,
    Ubigint,
    Boolean,
    Text,
}

fn to_duckdb_type(xsdType: &XsdType) -> DuckDBType {
    match xsdType {
        XsdType::Date => DuckDBType::Date,
        XsdType::DateTime => DuckDBType::Timestamp,
        XsdType::Float => DuckDBType::Float,
        XsdType::Double => DuckDBType::Double,
        XsdType::Decimal => DuckDBType::Double,
        XsdType::Byte => DuckDBType::Tinyint,
        XsdType::Short => DuckDBType::Smallint,
        XsdType::Int => DuckDBType::Integer,
        XsdType::NonNegativeInteger => DuckDBType::Ubigint,
        XsdType::Boolean => DuckDBType::Boolean,
        XsdType::String => DuckDBType::Text,
    }
}

fn duckdb_type_to_string(duckDBType: DuckDBType) -> &'static str {
    match duckDBType {
        DuckDBType::Date => "date",
        DuckDBType::Timestamp => "timestamp",
        DuckDBType::Float => "float",
        DuckDBType::Double => "double",
        DuckDBType::Tinyint => "tinyint",
        DuckDBType::Smallint => "smallint",
        DuckDBType::Integer => "integer",
        DuckDBType::Ubigint => "ubigint",
        DuckDBType::Boolean => "boolean",
        DuckDBType::Text => "text",
    }
}

#[derive(Serialize, Deserialize)]
struct Field<'a> {
    name: &'a str,
    index: bool,
    xsd: XsdType,
    references: Option<Reference<'a>>,
}

fn to_duckdb_schema(field: &Field) -> String {
    let duckdb_type = duckdb_type_to_string(to_duckdb_type(&field.xsd));
    format!("{} {}", field.name, duckdb_type)
}

#[derive(Serialize, Deserialize)]
struct Spec<'a> {
    root: &'a str,
    element: &'a str,
    without_rowid: bool,
    primary: Option<&'a str>,
    fields: HashMap<&'a str, Field<'a>>,
}

fn duckdb_schema<'a>(spec: Spec<'a>) -> String {
    let columns = spec
        .fields
        .values()
        .map(|field| to_duckdb_schema(field))
        .collect::<Vec<_>>()
        .join(",\n    ");
    let primary = if let Some(primary) = spec.primary {
        &format!(",\nprimary key (\"{}\")", primary)
    } else {
        ""
    };
    format!(
        "create table if not exists \"{}\" (\n{}{}))",
        spec.element, columns, primary
    )
}

#[derive(Debug)]
enum ParserState {
    StartRoot,
    StartElementOrEndRoot,
    StartAttrOrEndElement,
    AttrCdataOrEndAttr(String),
    Done,
}

fn parse<'a>(spec: Spec<'a>, mut reader: Reader<&[u8]>) -> HashMap<String, Vec<String>> {
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
    println!("Hello, world!");
}
