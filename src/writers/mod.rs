use std::error::Error;

use crate::transaction::Transaction;

pub mod csv;

pub trait Writer {
    fn write(transactions: &Vec<Transaction>) -> Result<String, Box<dyn Error>>;
}
