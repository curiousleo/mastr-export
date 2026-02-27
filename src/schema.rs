use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum XsdType {
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

impl XsdType {
    /// Returns a parquet message type field definition line.
    pub fn parquet_type_str(&self, name: &str) -> String {
        match self {
            XsdType::Date => format!("  OPTIONAL INT32 {} (DATE);", name),
            XsdType::DateTime => format!("  OPTIONAL INT64 {} (TIMESTAMP(MICROS,false));", name),
            XsdType::Float => format!("  OPTIONAL FLOAT {};", name),
            XsdType::Double => format!("  OPTIONAL DOUBLE {};", name),
            XsdType::Byte => format!("  OPTIONAL INT32 {} (INTEGER(8,true));", name),
            XsdType::Short => format!("  OPTIONAL INT32 {} (INTEGER(16,true));", name),
            XsdType::Int => format!("  OPTIONAL INT32 {};", name),
            XsdType::NonNegativeInteger => {
                format!("  OPTIONAL INT64 {} (INTEGER(64,false));", name)
            }
            XsdType::Boolean => format!("  OPTIONAL BOOLEAN {};", name),
            XsdType::String => format!("  OPTIONAL BINARY {} (UTF8);", name),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(default)]
    pub xsd: XsdType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    pub root: String,
    pub element: String,
    #[serde(default)]
    pub without_rowid: bool,
    #[serde(default)]
    pub primary: Option<String>,
    pub fields: Vec<Field>,
}

impl Schema {
    pub fn get_metadata(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("root".to_string(), self.root.clone());
        map.insert("element".to_string(), self.element.clone());
        if self.without_rowid {
            map.insert("without_rowid".to_string(), "true".to_string());
        }
        if let Some(primary) = self.primary.clone() {
            map.insert("primary".to_string(), primary);
        }
        map
    }

    pub fn to_parquet_message_type(&self) -> String {
        let mut msg = "message schema {\n".to_string();
        for field in &self.fields {
            msg.push_str(&field.xsd.parquet_type_str(&field.name));
            msg.push('\n');
        }
        msg.push('}');
        msg
    }

    // pub fn load_from_file(path: &std::path::Path) -> Result<Self> {
    //     let schema_file = std::fs::File::open(path)
    //         .map_err(|e| anyhow!("Failed to open schema file {:?}: {}", path, e))?;
    //     let schema: Schema = serde_json::from_reader(schema_file)
    //         .map_err(|e| anyhow!("Failed to parse schema JSON: {}", e))?;
    //     Ok(schema)
    // }
}

pub fn load_schema_from_file(path: &PathBuf, primary: Option<String>) -> Result<Schema> {
    let schema_file = std::fs::File::open(path)
        .map_err(|e| anyhow!("Failed to open schema file {:?}: {}", path, e))?;
    let mut schema: Schema = serde_json::from_reader(schema_file)
        .map_err(|e| anyhow!("Failed to parse schema JSON: {}", e))?;
    schema.primary = primary;
    Ok(schema)
}
