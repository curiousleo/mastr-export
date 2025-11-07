mod parser;
mod schema;

use anyhow::{Context, Result, anyhow};
use argh::FromArgs;
use std::{ffi::OsStr, path::PathBuf};

use parser::XmlParser;
use schema::load_schemas_from_file;

/// Marktstammdatenregister ZIP archive converter
#[derive(Debug, FromArgs)]
struct Args {
    #[argh(option)]
    /// root of the schema files
    schemas: PathBuf,

    #[argh(option)]
    /// path to the Marktstammdatenregister ZIP archive
    zip: PathBuf,
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let schemas = load_schemas_from_file(&args.schemas)?;

    let zip_file = std::fs::File::open(&args.zip)
        .map_err(|e| anyhow!("Failed to open ZIP file {:?}: {}", &args.zip, e))?;
    let mut archive =
        zip::ZipArchive::new(zip_file).map_err(|e| anyhow!("Failed to read ZIP archive: {}", e))?;
    let file_names = archive
        .file_names()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();

    for name in file_names {
        let (prefix, _) = name
            .split_once(&['_', '.'])
            .ok_or_else(|| anyhow!("XML file name does not contain '_' or '.': {}", name))?;
        let schema = schemas.get(OsStr::new(prefix)).ok_or_else(|| {
            anyhow!(
                "No schema found for this file: {} (prefix: {})",
                name,
                prefix
            )
        })?;
        let zip_file = archive
            .by_name(&name)
            .map_err(|e| anyhow!("Failed to read XML file '{}' in archive: {}", name, e))?;
        let buf_reader = std::io::BufReader::with_capacity(1024 * 1024, zip_file);
        let xml_reader = XmlParser::create_reader(buf_reader);
        let record_batch = XmlParser::parse(schema, xml_reader)
            .context(format!("Failed to parse XML file {}", name))?;
        println!(
            "{}: {} x {}",
            name,
            record_batch.num_rows(),
            record_batch.num_columns(),
        );
    }
    Ok(())
}
