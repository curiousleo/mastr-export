mod parser;
mod schema;

use anyhow::{Result, anyhow};
use std::path::PathBuf;

use parser::XmlParser;
use schema::Schema;

/// Load and parse XML data from a ZIP archive according to a schema
fn process_xml_from_zip(
    schema: &Schema,
    zip_path: &PathBuf,
    xml_name: &str,
) -> Result<arrow::array::RecordBatch> {
    let zip_file = std::fs::File::open(zip_path)
        .map_err(|e| anyhow!("Failed to open ZIP file {:?}: {}", zip_path, e))?;

    let mut archive =
        zip::ZipArchive::new(zip_file).map_err(|e| anyhow!("Failed to read ZIP archive: {}", e))?;

    let file = archive
        .by_name(xml_name)
        .map_err(|e| anyhow!("Failed to find XML file '{}' in archive: {}", xml_name, e))?;

    let buf_reader = std::io::BufReader::new(file);
    let xml_reader = XmlParser::create_reader(buf_reader);

    XmlParser::parse(schema, xml_reader)
}

/// Parse command line arguments and return the required paths and XML filename
fn parse_args() -> Result<(PathBuf, PathBuf, String)> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        return Err(anyhow!(
            "Usage: {} <schema.json> <archive.zip> <xml_file_name>",
            args.get(0).unwrap_or(&"program".to_string())
        ));
    }

    let schema_path = PathBuf::from(&args[1]);
    let zip_path = PathBuf::from(&args[2]);
    let xml_name = args[3].clone();

    Ok((schema_path, zip_path, xml_name))
}

// TODO: Use clap or similar for proper CLI argument parsing
fn main() -> Result<()> {
    let (schema_path, zip_path, xml_name) = parse_args()?;

    // Load schema from file
    let schema = Schema::load_from_file(&schema_path)?;

    // Display schema information
    println!("Schema: {:?}", arrow::datatypes::Schema::from(&schema));

    // Process XML file from ZIP archive
    let record_batch = process_xml_from_zip(&schema, &zip_path, &xml_name)?;

    // Display results
    println!("Number of columns: {}", record_batch.num_columns());
    println!("Number of rows: {}", record_batch.num_rows());

    Ok(())
}
