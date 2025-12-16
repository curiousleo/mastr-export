mod parser;
mod schema;

use anyhow::{Context, Result};
use argh::FromArgs;
use parquet::{
    arrow::ArrowWriter,
    basic::ZstdLevel,
    file::{metadata::KeyValue, properties::WriterProperties},
};
use std::{fs::File, io::BufWriter, path::PathBuf};

use parser::XmlParser;
use schema::load_schema_from_file;

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
    let record_batch = XmlParser::parse(&schema, buf_reader).context("Failed to parse XML file")?;

    let writer = BufWriter::new(File::create(args.output)?);

    let key_value_metadata = record_batch
        .schema()
        .metadata()
        .clone()
        .into_iter()
        .map(|(k, v)| KeyValue::new(k, v))
        .collect::<Vec<_>>();
    let parquet_writer_props = WriterProperties::builder()
        .set_compression(parquet::basic::Compression::ZSTD(ZstdLevel::default()))
        .set_key_value_metadata(Some(key_value_metadata))
        .build();
    let mut arrow_writer =
        ArrowWriter::try_new(writer, record_batch.schema(), Some(parquet_writer_props))?;
    arrow_writer.write(&record_batch)?;
    arrow_writer.close()?;
    // println!(
    //     "{} x {}",
    //     record_batch.num_rows(),
    //     record_batch.num_columns(),
    // );
    Ok(())
}
