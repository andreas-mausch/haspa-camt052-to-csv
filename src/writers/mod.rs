use std::error::Error;
use std::io::Write;

use crate::transaction::Transaction;

pub mod csv;

pub trait Writer {
    fn write<W: Write>(transactions: &Vec<Transaction>, write: W) -> Result<(), Box<dyn Error>>;
}
