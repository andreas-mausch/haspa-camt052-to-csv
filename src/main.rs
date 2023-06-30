use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

use clap::Parser;
use env_logger::{Builder, Env};
use log::{debug, error, info, warn};

use crate::transaction::Transaction;
use crate::writers::csv::Csv;
use crate::writers::Writer;
use crate::xml_document_finder::XmlDocumentFinder;

mod xml_document_finder;
mod my_money;
mod iso_date;
mod transaction;
mod writers;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[structopt(index = 1, required = true)]
    files: Vec<String>,

    #[arg(short, long, default_value = "-")]
    output: String,
}

fn process_xml<'a, R: Read>(mut reader: R) -> Result<Vec<Transaction<'a>>, Box<dyn Error>> {
    let mut xml_content = String::new();
    reader.read_to_string(&mut xml_content)?;

    let document = roxmltree::Document::parse(&xml_content)?;
    let root = document.root();
    let entries = root.filter("Document/BkToCstmrAcctRpt/Rpt/Ntry");

    if entries.is_empty() {
        warn!("No entries found");
        return Ok(vec![]);
    }

    let transactions: Vec<_> = entries.iter().map(|entry| entry.try_into()).collect();
    transactions.into_iter().collect()
}

fn process_file<'a, R: Read + Seek>(path: &Path, read: R) -> Result<Vec<Transaction<'a>>, Box<dyn Error>> {
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
            Ok(vec![])
        }
    }.map_err(|e| format!("Error processing file {:?}: {}", path, e.to_string()).into())
}

fn read_zip<'a, R: Read + Seek>(path: &Path, reader: R) -> Result<Vec<Transaction<'a>>, Box<dyn Error>> {
    let mut archive = zip::ZipArchive::new(reader)?;
    let transactions = (0..archive.len()).map(|index| {
        let mut file_in_archive = archive.by_index(index)?;
        debug!("File in archive: {:?}", file_in_archive.name());

        let mut buffer = Vec::new();
        file_in_archive.read_to_end(&mut buffer)?;
        let cursor = Cursor::new(buffer);
        process_file(path.join(file_in_archive.name()).as_path(), cursor)
    }).collect::<Result<Vec<_>, _>>()?
        .into_iter().flatten().collect();
    Ok(transactions)
}

fn main() {
    Builder::from_env(Env::default().default_filter_or("debug")).init();

    let args = Args::parse();
    info!("Files {:?}!", args.files);
    let paths = args.files.iter().map(|file| Path::new(file)).collect::<Vec<_>>();
    let non_existing_files = paths.iter().filter(|path| !path.exists() || !path.is_file()).collect::<Vec<_>>();

    if !non_existing_files.is_empty() {
        error!("File does not exist: {:?}", non_existing_files);
        return;
    }

    info!("All files exist: {:?}", args.files);

    let transactions: Vec<_> = paths.iter().flat_map(|path| {
        File::open(path)
            .map_err(|e| e.into())
            .and_then(|f| {
                let reader = BufReader::new(f);
                process_file(path, reader)
            })
            .expect("Could not read file")
    }).collect();

    let csv_output = Csv::write(&transactions).expect("Cannot serialise to .csv");
    // Replace File::create() by File::create_new() once it is stable
    let mut output_stream: Box<dyn Write> = if args.output == "-" { Box::new(io::stdout()) } else { Box::new(File::create(args.output).unwrap()) };

    writeln!(output_stream, "{}", csv_output).unwrap();
}
