use std::error::Error;
use std::io::Write;

use crate::transaction::Transaction;

pub mod csv;
pub mod ods;

pub trait Writer {
    fn write<W: Write>(transactions: &[Transaction], write: W) -> Result<(), Box<dyn Error>>;
}
