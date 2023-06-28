use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

use chrono::NaiveDate;
use clap::Parser;
use csv::WriterBuilder;
use env_logger::{Builder, Env};
use iban::Iban;
use log::{debug, error, info, warn};
use roxmltree::Node;
use rust_decimal::Decimal;
use rusty_money::{iso, Money};
use rusty_money::iso::Currency;
use serde::Serialize;

use crate::date_parser::IsoDate;
use crate::rusty_money_serde::MyMoney;
use crate::xml_document_finder::XmlDocumentFinder;

mod xml_document_finder;
mod rusty_money_serde;
mod date_parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[structopt(index = 1, required = true)]
    files: Vec<String>,

    #[arg(short, long, default_value = "-")]
    output: String,
}

#[derive(Debug, Serialize)]
struct Party {
    name: String,
    iban: Option<Iban>,
}

#[derive(Debug, Serialize)]
struct Transaction<'a> {
    date: NaiveDate,
    valuta: NaiveDate,
    amount: MyMoney<'a, Currency>,
    creditor: Party,
    debtor: Party,
    transaction_type: String,
    description: String,
}

impl TryFrom<&Node<'_, '_>> for Transaction<'_> {
    type Error = Box<dyn Error>;

    fn try_from(value: &Node) -> Result<Self, Self::Error> {
        let date = value.get_into::<IsoDate>("BookgDt/Dt")?.0;
        let valuta = value.get_into::<IsoDate>("ValDt/Dt")?.0;
        let debit = value.get_into::<String>("CdtDbtInd")? == "DBIT";
        let amount = value.get_into::<String>("Amt")?;
        let currency = value.find("Amt")
            .and_then(|it| it.attribute("Ccy"))
            .ok_or::<Box<dyn Error>>("No text in 'Amt[Ccy]' attribute".into())?;
        let creditor = value.find_into::<String>("NtryDtls/TxDtls/RltdPties/Cdtr/Nm")?
            .or(value.find_into::<String>("NtryDtls/TxDtls/RltdPties/Cdtr/Pty/Nm")?)
            .unwrap_or_else(|| {
                warn!("No creditor found: Date {}, Amount {}", date, amount);
                "".to_string()
            }).trim().to_string();
        let creditor_iban = value.find_into::<Iban>("NtryDtls/TxDtls/RltdPties/CdtrAcct/Id/IBAN")?;
        let debtor = value.find_into::<String>("NtryDtls/TxDtls/RltdPties/Dbtr/Nm")?
            .or(value.find_into::<String>("NtryDtls/TxDtls/RltdPties/Dbtr/Pty/Nm")?)
            .unwrap_or_else(|| {
                warn!("No debtor found: Date {}, Amount {}", date, amount);
                "".to_string()
            }).trim().to_string();
        let debtor_iban = value.find("NtryDtls/TxDtls/RltdPties/DbtrAcct/Id/IBAN")
            .and_then(|it| it.text())
            .map(|node| node.trim())
            .and_then(|iban| iban.parse::<Iban>().ok());
        let transaction_type = value.get_into::<String>("AddtlNtryInf")?.trim().to_string();
        let description = value.filter("NtryDtls/TxDtls/RmtInf/Ustrd")
            .iter().map(|node| node.text().unwrap_or(""))
            .map(|node| node.trim())
            .collect::<Vec<_>>().join("; ");

        // rusty_money sets the locale on the currency EUR
        // and expects it to be formatted like
        // 1.000,00 and not like 1,000.00
        // https://github.com/varunsrin/rusty_money/issues/61
        // That's why we need to convert the String to a Decimal first, and then call rusty_money.
        // Otherwise, we could use Money::from_str() directly.
        let money_decimal = amount.parse::<Decimal>()
            .map(|amount| if debit { -amount } else { amount })?;
        let money = MyMoney(Money::from_decimal(money_decimal,
                                                iso::find(currency).ok_or("Currency not found")?));

        Ok(Transaction {
            date,
            valuta,
            amount: money,
            creditor: Party { name: creditor.to_string(), iban: creditor_iban },
            debtor: Party { name: debtor.to_string(), iban: debtor_iban },
            transaction_type: transaction_type.to_string(),
            description,
        })
    }
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

fn to_csv(transactions: Vec<Transaction>) -> Result<String, Box<dyn Error>> {
    let mut writer = WriterBuilder::new().has_headers(true).delimiter(b';').from_writer(vec![]);
    writer.serialize(("Date", "Valuta", "Amount", "Currency", "Creditor Name", "Creditor IBAN", "Debtor Name", "Debtor IBAN", "Transaction Type", "Description"))?;
    transactions.iter().try_for_each(|transaction| {
        writer.serialize(transaction)
    })?;
    writer.flush()?;
    Ok(String::from_utf8(writer.into_inner()?)?)
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

    let csv_output = to_csv(transactions).expect("Cannot serialise to .csv");
    // Replace File::create() by File::create_new() once it is stable
    let mut output_stream: Box<dyn Write> = if args.output == "-" { Box::new(io::stdout()) } else { Box::new(File::create(args.output).unwrap()) };

    writeln!(output_stream, "{}", csv_output).unwrap();
}
