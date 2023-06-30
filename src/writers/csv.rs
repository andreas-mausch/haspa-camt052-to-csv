use std::error::Error;

use csv::WriterBuilder;

use crate::transaction::Transaction;
use crate::writers::Writer;

pub struct Csv {}

impl Writer for Csv {
    fn write(transactions: &Vec<Transaction>) -> Result<String, Box<dyn Error>> {
        let mut writer = WriterBuilder::new().has_headers(true).delimiter(b';').from_writer(vec![]);
        writer.serialize(("Date", "Valuta", "Amount", "Currency", "Creditor Name", "Creditor IBAN", "Debtor Name", "Debtor IBAN", "Transaction Type", "Description"))?;
        transactions.iter().try_for_each(|transaction| {
            writer.serialize(transaction)
        })?;
        writer.flush()?;
        Ok(String::from_utf8(writer.into_inner()?)?)
    }
}
