use crate::error::Error;
use crate::Transaction;
use std::collections::HashMap;
use std::io::{BufRead, Write};

const TX_ID: &str = "TX_ID";
const TX_TYPE: &str = "TX_TYPE";
const FROM_USER_ID: &str = "FROM_USER_ID";
const TO_USER_ID: &str = "TO_USER_ID";
const AMOUNT: &str = "AMOUNT";
const TIMESTAMP: &str = "TIMESTAMP";
const STATUS: &str = "STATUS";
const DESCRIPTION: &str = "DESCRIPTION";

pub fn read<B: BufRead>(reader: B) -> Result<Vec<Transaction>, Error> {
    let mut transactions = Vec::new();
    let mut tx_map = HashMap::new();
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if line.starts_with('#') {
                    continue;
                }
                if line.trim().is_empty() && !tx_map.is_empty() {
                    transactions.push(Transaction::try_from(tx_map)?);
                    tx_map = HashMap::new();
                    continue;
                }

                if let Some((key, value)) = line.split_once(':') {
                    tx_map.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
            Err(err) => return Err(Error::Parse(format!("read line: {:?}", err))),
        }
    }
    if !tx_map.is_empty() {
        transactions.push(Transaction::try_from(tx_map)?);
    }

    Ok(transactions)
}

pub fn write<W: Write>(w: &mut W, transactions: Vec<Transaction>) -> Result<(), Error> {
    for tx in transactions {
        writeln!(w, "{}: {}", TX_ID, tx.tx_id).
            map_err(|e| Error::Write(format!("write tx_id: {:?}", e)))?;
        writeln!(w, "{}: {}", TX_TYPE, tx.tx_type).
            map_err(|e| Error::Write(format!("write tx_type: {:?}", e)))?;
        writeln!(w, "{}: {}", FROM_USER_ID, tx.from_user_id).
            map_err(|e| Error::Write(format!("write from_user_id: {:?}", e)))?;
        writeln!(w, "{}: {}", TO_USER_ID, tx.to_user_id).
            map_err(|e| Error::Write(format!("write to_user_id: {:?}", e)))?;
        writeln!(w, "{}: {}", AMOUNT, tx.amount).
            map_err(|e| Error::Write(format!("write amount: {:?}", e)))?;
        writeln!(w, "{}: {}", TIMESTAMP, tx.timestamp).
            map_err(|e| Error::Write(format!("write timestamp: {:?}", e)))?;
        writeln!(w, "{}: {}", STATUS, tx.status).
            map_err(|e| Error::Write(format!("write status: {:?}", e)))?;
        writeln!(w, "{}: {}", DESCRIPTION, tx.description).
            map_err(|e| Error::Write(format!("write description: {:?}", e)))?;
        write!(w, "\n").map_err(|e| Error::Write(format!("write empty line: {:?}", e)))?;
    }
    Ok(())
}

impl TryFrom<HashMap<String, String>> for Transaction {
    type Error = Error;

    fn try_from(map: HashMap<String, String>) -> Result<Self, Self::Error> {
        if map.len() != 8 {
            return Err(Error::Parse(format!("field count: {:?}", map)));
        };
        Ok(Transaction {
            tx_id: map[TX_ID].parse().map_err(|e| {
                Error::Parse(format!("tx_id parse: {:?}", e))
            })?,
            tx_type: map[TX_TYPE].parse().map_err(|e| {
                Error::Parse(format!("tx_type: {:?}", e))
            })?,
            from_user_id: map[FROM_USER_ID].parse().map_err(|e| {
                Error::Parse(format!("from_user_id: {:?}", e))
            })?,
            to_user_id: map[TO_USER_ID].parse().map_err(|e| {
                Error::Parse(format!("to_user_id: {:?}", e))
            })?,
            amount: map[AMOUNT].parse().map_err(|e| {
                Error::Parse(format!("amount: {:?}", e))
            })?,
            timestamp: map[TIMESTAMP].parse().map_err(|e| {
                Error::Parse(format!("timestamp: {:?}", e))
            })?,
            status: map[STATUS].parse().map_err(|e| {
                Error::Parse(format!("status: {:?}", e))
            })?,
            description: map[DESCRIPTION].to_string(),
        })
    }
}