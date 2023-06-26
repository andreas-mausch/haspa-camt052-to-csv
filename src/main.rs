use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::Path;

use clap::Parser;
use env_logger::{Builder, Env};
use log::{debug, error, info, warn};

use crate::xml_document_finder::XmlDocumentFinder;

mod xml_document_finder;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[structopt(index = 1, required = true)]
    files: Vec<String>,
}

fn process_xml<R: Read>(mut reader: R) -> Result<(), Box<dyn Error>> {
    let mut xml_content = String::new();
    reader.read_to_string(&mut xml_content)?;

    let document = roxmltree::Document::parse(&xml_content)?;
    let root = document.root();
    let entries = root.filter("Document/BkToCstmrAcctRpt/Rpt/Ntry");

    if entries.is_empty() {
        warn!("No entries found");
        return Ok(());
    }

    entries.iter().for_each(|entry| {
        let debit = entry.find("CdtDbtInd").and_then(|it| it.text()) == Some("DBIT");
        let amount = entry.find("Amt").and_then(|it| it.text());
        let currency = entry.find("Amt").and_then(|it| it.attribute("Ccy"));
        info!("Debit {:?}, Amount {:?} {:?}", debit, amount, currency);
    });

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
            info!("Processing XML file: {:?}", path);
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
    Builder::from_env(Env::default().default_filter_or("debug")).init();

    let args = Args::parse();
    info!("Files {:?}!", args.files);
    let paths: Vec<_> = args.files.iter().map(|file| Path::new(file)).collect();

    let non_existing_files: Vec<_> = paths.iter().filter(|path| !path.exists() || !path.is_file()).collect();

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
