use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::Path;

use clap::Parser;
use env_logger::Env;
use log::{debug, error, info, warn};
use sxd_document::parser;
use sxd_xpath::{Context, evaluate_xpath, Factory, Value};
use sxd_xpath::nodeset::Nodeset;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[structopt(index = 1, required = true)]
    files: Vec<String>,
}

trait ValueExt {
    fn to_nodeset(&self) -> Result<&Nodeset, &'static str>;
}

impl ValueExt for Value<'_> {
    fn to_nodeset(&self) -> Result<&Nodeset, &'static str> {
        match self {
            Value::Nodeset(nodes) => Ok(nodes),
            _ => Err("Value is not a nodeset: {:?}")
        }
    }
}

fn process_xml<R: Read>(mut reader: R) -> Result<(), Box<dyn Error>> {
    let mut xml_content = String::new();
    reader.read_to_string(&mut xml_content)?;

    let package = parser::parse(&xml_content)?;
    let document = package.as_document();

    // First way: Set the namespace explicitly. Bulky.
    let factory = Factory::new();
    let xpath = factory.build("/ns:Document")?.ok_or("Could not compile XPath")?;
    let mut context = Context::new();
    // We need to define a namespace ourselves and use it.
    // There don't seem to be support for the default namespace.
    // See also here: https://github.com/shepmaster/sxd-xpath/issues/133
    context.set_namespace("ns", "urn:iso:std:iso:20022:tech:xsd:camt.052.001.02");
    let value1 = xpath.evaluate(&context, document.root())
        .expect("XPath evaluation failed");

    // Second way: Get the value by local-name only. Longer xpath expression.
    let value2 = evaluate_xpath(&document, "/*[local-name() = 'Document']")?;

    let entries = value2.to_nodeset()?;
    info!("Entries found: {:?}", entries);

    Ok(())
}

fn process_file<R: Read + Seek>(path: &Path, read: R) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(read);
    let mut beginning_of_file = vec![0u8; 2048];
    reader.read(&mut beginning_of_file)?;
    reader.seek(SeekFrom::Start(0))?;

    match tree_magic_mini::from_u8(&beginning_of_file) {
        "application/zip" => {
            info!("Zip file: {:?}", path);
            read_zip(path, reader)
        }
        "application/xml" => {
            process_xml(reader)
        }
        _ => {
            warn!("File found, but it is not ZIP or XML, skipping: {:?}", path);
            Ok(())
        }
    }
}

fn read_zip<R: Read + Seek>(path: &Path, reader: R) -> Result<(), Box<dyn Error>> {
    let mut archive = zip::ZipArchive::new(reader)?;
    for index in 0..archive.len() {
        let mut file_in_archive = archive.by_index(index)?;
        debug!("File in archive: {:?}", file_in_archive.name());

        let mut buffer = Vec::new();
        file_in_archive.read_to_end(&mut buffer)?;
        let cursor = Cursor::new(buffer);
        process_file(path.join(file_in_archive.name()).as_path(), cursor)?
    }
    Ok(())
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let args = Args::parse();
    info!("Files {:?}!", args.files);
    let paths: Vec<_> = args.files.iter().map(|file| Path::new(file)).collect();

    let non_existing_files: Vec<_> = paths.iter().filter(|path| { !path.exists() || !path.is_file() }).collect();

    if !non_existing_files.is_empty() {
        error!("File does not exist: {:?}", non_existing_files);
        return;
    }

    info!("All files exist: {:?}", args.files);

    paths.iter().for_each(|path| {
        File::open(path)
            .map_err(|e| e.into())
            .and_then(|f| {
                let reader = BufReader::new(f);
                process_file(path, reader)
            })
            .expect("Could not read file");
    });
}
