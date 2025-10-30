use anyhow::Result;
use arrow::datatypes::{DataType, TimeUnit};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub struct Field {
    pub name: String,
    #[serde(default)]
    pub xsd: XsdType,
}

impl From<&Field> for arrow::datatypes::Field {
    fn from(field: &Field) -> Self {
        let data_type = DataType::from(field.xsd);
        arrow::datatypes::Field::new(&field.name, data_type, true)
    }
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

    pub fn load_from_file(path: &std::path::Path) -> Result<Self> {
        let schema_file = std::fs::File::open(path)
            .map_err(|e| anyhow::anyhow!("Failed to open schema file {:?}: {}", path, e))?;
        let schema: Schema = serde_json::from_reader(schema_file)
            .map_err(|e| anyhow::anyhow!("Failed to parse schema JSON: {}", e))?;
        Ok(schema)
    }
}

impl From<&Schema> for arrow::datatypes::Schema {
    fn from(schema: &Schema) -> Self {
        let fields = schema
            .fields
            .iter()
            .map(|field| arrow::datatypes::Field::from(field))
            .collect::<Vec<_>>();
        let fields = arrow::datatypes::Fields::from(fields);
        arrow::datatypes::Schema::new(fields).with_metadata(schema.get_metadata())
    }
}
