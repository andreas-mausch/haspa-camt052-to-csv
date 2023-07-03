use std::error::Error;
use std::io::Write;

use csv::WriterBuilder;

use crate::transaction::Transaction;
use crate::writers::Writer;

pub struct Csv;

impl Writer for Csv {
    fn write<W: Write>(transactions: &[Transaction], write: W) -> Result<(), Box<dyn Error>> {
        let mut writer = WriterBuilder::new().has_headers(true).delimiter(b';').from_writer(write);
        writer.serialize(("Date", "Valuta", "Amount", "Currency", "Creditor Name", "Creditor IBAN", "Debtor Name", "Debtor IBAN", "Transaction Type", "Description"))?;
        transactions.iter().try_for_each(|transaction| {
            writer.serialize(transaction)
                .map_err(|e| e.into())
                .and_then(|()| writer.flush()
                    .map_err(|e| e.into()))
        })
    }
}
