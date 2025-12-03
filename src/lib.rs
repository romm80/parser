mod bin_format;
mod csv_format;
mod error;
mod txt_format;

use error::Error;
use std::fmt::Display;
use std::io::{BufReader, BufWriter, Read, Write};

pub enum Format {
    Bin,
    Csv,
    Txt,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Transaction {
    pub tx_id: u64,
    pub tx_type: TransactionType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: u64,
    pub timestamp: u64,
    pub status: Status,
    pub description: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TransactionType {
    DEPOSIT,
    TRANSFER,
    WITHDRAWAL,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Status {
    SUCCESS,
    FAILURE,
    PENDING,
}

impl Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::DEPOSIT => {
                write!(f, "DEPOSIT")
            }
            TransactionType::TRANSFER => {
                write!(f, "TRANSFER")
            }
            TransactionType::WITHDRAWAL => {
                write!(f, "WITHDRAWAL")
            }
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::SUCCESS => {
                write!(f, "SUCCESS")
            }
            Status::FAILURE => {
                write!(f, "FAILURE")
            }
            Status::PENDING => {
                write!(f, "PENDING")
            }
        }
    }
}

pub fn read<R: Read>(data: R, format: Format) -> Result<Vec<Transaction>, Error> {
    let reader = BufReader::new(data);
    match format {
        Format::Csv => csv_format::read(reader),
        Format::Bin => bin_format::read(reader),
        Format::Txt => txt_format::read(reader),
    }
}

pub fn write<W: Write>(data: W, format: Format, tx: Vec<Transaction>) -> Result<(), Error> {
    let mut buf = BufWriter::new(data);
    match format {
        Format::Csv => csv_format::write(&mut buf, tx)?,
        Format::Bin => bin_format::write(&mut buf, tx)?,
        Format::Txt => txt_format::write(&mut buf, tx)?,
    }
    buf.flush()
        .map_err(|e| Error::Write(format!("flush buf: {:?}", e)))
}
