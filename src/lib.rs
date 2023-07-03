use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

use clap::{Parser, ValueEnum};
use log::{debug, error, info, warn};

use writers::csv::Csv;
use writers::ods::Ods;

use crate::transaction::Transaction;
use crate::writers::Writer;
use crate::xml_document_finder::XmlDocumentFinder;

mod xml_document_finder;
mod my_money;
mod iso_date;
mod transaction;
mod writers;

#[derive(ValueEnum, Clone, Debug)]
pub enum Format {
    Csv,
    Ods,
}

/// Convert camt052 files into csv or ods
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Input camt052 files
    #[arg(required = true)]
    files: Vec<String>,

    /// Format of the output file
    #[arg(value_enum, short, long, default_value_t = Format::Csv)]
    format: Format,

    /// Output filename. Use "-" to output to stdout
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

pub fn process(files: Vec<String>, format: Format, output_stream: &mut dyn Write) -> Result<(), Box<dyn Error>> {
    info!("Files {:?}!", files);
    let paths = files.iter().map(|file| Path::new(file)).collect::<Vec<_>>();
    let non_existing_files = paths.iter().filter(|path| !path.exists() || !path.is_file()).collect::<Vec<_>>();

    if !non_existing_files.is_empty() {
        return Err(format!("File does not exist: {:?}", non_existing_files).into());
    }

    info!("All files exist: {:?}", files);

    let transactions: Vec<_> = paths.iter().map(|path| {
        File::open(path)
            .map_err(|e| e.into())
            .and_then(|f| {
                let reader = BufReader::new(f);
                process_file(path, reader)
            })
    }).collect();
    let y: Result<Vec<Vec<Transaction>>, Box<dyn Error>> = transactions.into_iter().collect();
    let z: Result<Vec<Transaction>, Box<dyn Error>> = y.map(|value| value.into_iter().flatten().collect());

    let write = match format {
        Format::Csv => Csv::write,
        Format::Ods => Ods::write
    };
    write(&z?, output_stream)
}

fn get_output_stream(output: &str) -> Result<Box<dyn Write>, Box<dyn Error>> {
    if output == "-" {
        Ok(Box::new(io::stdout()))
    } else {
        // Replace File::create() by File::create_new() once it is stable
        let x: Result<Box<dyn Write>, Box<dyn Error>> = File::create(output)
            .map(|file| -> Box<dyn Write> { Box::new(file) })
            .map_err(|e| e.into());
        x
    }
}

pub fn camt052(args: Args) {
    get_output_stream(&args.output).and_then(|mut output_stream|
        process(args.files, args.format, &mut output_stream))
        .unwrap_or_else(|e| {
            error!("Could not parse files {:#?}", e);
            std::process::exit(1)
        });
}
