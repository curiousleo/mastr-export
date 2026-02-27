mod parser;
mod schema;

use anyhow::{Context, Result, anyhow};
use argh::FromArgs;
use chrono::NaiveDate;
use parquet::{
    basic::ZstdLevel,
    data_type::{BoolType, ByteArray, ByteArrayType, DoubleType, FloatType, Int32Type, Int64Type},
    file::{
        metadata::KeyValue,
        properties::WriterProperties,
        writer::SerializedFileWriter,
    },
    schema::parser::parse_message_type,
};
use std::{fs::File, io::BufWriter, path::PathBuf, sync::Arc};

use parser::XmlParser;
use schema::{XsdType, load_schema_from_file};

/// Marktstammdatenregister ZIP archive converter
#[derive(Debug, FromArgs)]
struct Args {
    #[argh(option)]
    /// the schema file to parse with
    schema: PathBuf,

    #[argh(option)]
    /// path to the output file
    output: PathBuf,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let schema = load_schema_from_file(&args.schema, None)?;

    let buf_reader = std::io::BufReader::with_capacity(64 * 1024, std::io::stdin());
    let columns =
        XmlParser::parse(&schema, buf_reader).context("Failed to parse XML file")?;

    let message_type = schema.to_parquet_message_type();
    let parquet_schema = Arc::new(parse_message_type(&message_type)?);

    let key_value_metadata = schema
        .get_metadata()
        .into_iter()
        .map(|(k, v)| KeyValue::new(k, v))
        .collect::<Vec<_>>();
    let props = Arc::new(
        WriterProperties::builder()
            .set_compression(parquet::basic::Compression::ZSTD(ZstdLevel::default()))
            .set_key_value_metadata(Some(key_value_metadata))
            .build(),
    );

    let file = BufWriter::new(File::create(&args.output)?);
    let mut writer = SerializedFileWriter::new(file, parquet_schema, props)?;
    let mut row_group_writer = writer.next_row_group()?;

    for (i, col_data) in columns.iter().enumerate() {
        let mut col_writer = row_group_writer
            .next_column()?
            .ok_or_else(|| anyhow!("Expected column writer for field {}", i))?;

        write_column(&mut col_writer, col_data, schema.fields[i].xsd)
            .with_context(|| format!("Failed to write column '{}'", schema.fields[i].name))?;

        col_writer.close()?;
    }

    row_group_writer.close()?;
    writer.close()?;

    Ok(())
}

fn write_column(
    col_writer: &mut parquet::file::writer::SerializedColumnWriter<'_>,
    values: &[Option<String>],
    xsd_type: XsdType,
) -> Result<()> {
    match xsd_type {
        XsdType::Date => {
            let (vals, def_levels) = parse_int32_column(values, |s| {
                let date = NaiveDate::parse_from_str(&s[..10.min(s.len())], "%Y-%m-%d")
                    .map_err(|e| anyhow!("Failed to parse date '{}': {}", s, e))?;
                let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
                Ok((date - epoch).num_days() as i32)
            })?;
            col_writer
                .typed::<Int32Type>()
                .write_batch(&vals, Some(&def_levels), None)?;
        }
        XsdType::Byte => {
            let (vals, def_levels) = parse_int32_column(values, |s| {
                Ok(s.parse::<i8>()
                    .map_err(|e| anyhow!("Failed to parse byte '{}': {}", s, e))?
                    as i32)
            })?;
            col_writer
                .typed::<Int32Type>()
                .write_batch(&vals, Some(&def_levels), None)?;
        }
        XsdType::Short => {
            let (vals, def_levels) = parse_int32_column(values, |s| {
                Ok(s.parse::<i16>()
                    .map_err(|e| anyhow!("Failed to parse short '{}': {}", s, e))?
                    as i32)
            })?;
            col_writer
                .typed::<Int32Type>()
                .write_batch(&vals, Some(&def_levels), None)?;
        }
        XsdType::Int => {
            let (vals, def_levels) = parse_int32_column(values, |s| {
                s.parse::<i32>()
                    .map_err(|e| anyhow!("Failed to parse int '{}': {}", s, e))
            })?;
            col_writer
                .typed::<Int32Type>()
                .write_batch(&vals, Some(&def_levels), None)?;
        }
        XsdType::DateTime => {
            let (vals, def_levels) = parse_int64_column(values, |s| parse_datetime_micros(s))?;
            col_writer
                .typed::<Int64Type>()
                .write_batch(&vals, Some(&def_levels), None)?;
        }
        XsdType::NonNegativeInteger => {
            let (vals, def_levels) = parse_int64_column(values, |s| {
                Ok(s.parse::<u64>()
                    .map_err(|e| anyhow!("Failed to parse nonNegativeInteger '{}': {}", s, e))?
                    as i64)
            })?;
            col_writer
                .typed::<Int64Type>()
                .write_batch(&vals, Some(&def_levels), None)?;
        }
        XsdType::Float => {
            let mut vals = Vec::with_capacity(values.len());
            let mut def_levels = Vec::with_capacity(values.len());
            for opt in values {
                match opt {
                    Some(s) => {
                        vals.push(
                            s.parse::<f32>()
                                .map_err(|e| anyhow!("Failed to parse float '{}': {}", s, e))?,
                        );
                        def_levels.push(1);
                    }
                    None => def_levels.push(0),
                }
            }
            col_writer
                .typed::<FloatType>()
                .write_batch(&vals, Some(&def_levels), None)?;
        }
        XsdType::Double => {
            let mut vals = Vec::with_capacity(values.len());
            let mut def_levels = Vec::with_capacity(values.len());
            for opt in values {
                match opt {
                    Some(s) => {
                        vals.push(
                            s.parse::<f64>()
                                .map_err(|e| anyhow!("Failed to parse double '{}': {}", s, e))?,
                        );
                        def_levels.push(1);
                    }
                    None => def_levels.push(0),
                }
            }
            col_writer
                .typed::<DoubleType>()
                .write_batch(&vals, Some(&def_levels), None)?;
        }
        XsdType::Boolean => {
            let mut vals = Vec::with_capacity(values.len());
            let mut def_levels = Vec::with_capacity(values.len());
            for opt in values {
                match opt {
                    Some(s) => {
                        vals.push(match s.as_str() {
                            "true" | "1" => true,
                            "false" | "0" => false,
                            _ => return Err(anyhow!("Failed to parse boolean '{}'", s)),
                        });
                        def_levels.push(1);
                    }
                    None => def_levels.push(0),
                }
            }
            col_writer
                .typed::<BoolType>()
                .write_batch(&vals, Some(&def_levels), None)?;
        }
        XsdType::String => {
            let mut vals = Vec::with_capacity(values.len());
            let mut def_levels = Vec::with_capacity(values.len());
            for opt in values {
                match opt {
                    Some(s) => {
                        vals.push(ByteArray::from(s.as_str()));
                        def_levels.push(1);
                    }
                    None => def_levels.push(0),
                }
            }
            col_writer
                .typed::<ByteArrayType>()
                .write_batch(&vals, Some(&def_levels), None)?;
        }
    }
    Ok(())
}

fn parse_int32_column(
    values: &[Option<String>],
    parse_fn: impl Fn(&str) -> Result<i32>,
) -> Result<(Vec<i32>, Vec<i16>)> {
    let mut vals = Vec::with_capacity(values.len());
    let mut def_levels = Vec::with_capacity(values.len());
    for opt in values {
        match opt {
            Some(s) => {
                vals.push(parse_fn(s)?);
                def_levels.push(1);
            }
            None => def_levels.push(0),
        }
    }
    Ok((vals, def_levels))
}

fn parse_int64_column(
    values: &[Option<String>],
    parse_fn: impl Fn(&str) -> Result<i64>,
) -> Result<(Vec<i64>, Vec<i16>)> {
    let mut vals = Vec::with_capacity(values.len());
    let mut def_levels = Vec::with_capacity(values.len());
    for opt in values {
        match opt {
            Some(s) => {
                vals.push(parse_fn(s)?);
                def_levels.push(1);
            }
            None => def_levels.push(0),
        }
    }
    Ok((vals, def_levels))
}

fn parse_datetime_micros(s: &str) -> Result<i64> {
    use chrono::NaiveDateTime;

    // Try with fractional seconds first (most common in MaStR data)
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f") {
        return Ok(dt.and_utc().timestamp_micros());
    }
    // Try without fractional seconds
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt.and_utc().timestamp_micros());
    }
    // Try RFC 3339 (with timezone offset)
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Ok(dt.timestamp_micros());
    }
    Err(anyhow!("Failed to parse datetime '{}'", s))
}
