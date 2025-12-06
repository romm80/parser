use crate::Transaction;
use crate::error::{ParseError, WriteError};
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

struct FieldValue {
    value: String,
    line: usize,
}

pub fn read<B: BufRead>(reader: B) -> Result<Vec<Transaction>, ParseError> {
    let mut transactions = Vec::new();
    let mut tx_map = HashMap::new();

    for (idx, line) in reader.lines().enumerate() {
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
                    tx_map.insert(
                        key.trim().to_string(),
                        FieldValue {
                            value: value.trim().to_string(),
                            line: idx + 1,
                        },
                    );
                }
            }
            Err(e) => return Err(ParseError::Io(e)),
        }
    }
    if !tx_map.is_empty() {
        transactions.push(Transaction::try_from(tx_map)?);
    }

    Ok(transactions)
}

pub fn write<W: Write>(w: &mut W, transactions: Vec<Transaction>) -> Result<(), WriteError> {
    for tx in transactions {
        writeln!(w, "{}: {}", TX_ID, tx.tx_id)?;
        writeln!(w, "{}: {}", TX_TYPE, tx.tx_type)?;
        writeln!(w, "{}: {}", FROM_USER_ID, tx.from_user_id)?;
        writeln!(w, "{}: {}", TO_USER_ID, tx.to_user_id)?;
        writeln!(w, "{}: {}", AMOUNT, tx.amount)?;
        writeln!(w, "{}: {}", TIMESTAMP, tx.timestamp)?;
        writeln!(w, "{}: {}", STATUS, tx.status)?;
        writeln!(w, "{}: {}", DESCRIPTION, tx.description)?;
        writeln!(w)?;
    }
    Ok(())
}

impl TryFrom<HashMap<String, FieldValue>> for Transaction {
    type Error = ParseError;

    fn try_from(map: HashMap<String, FieldValue>) -> Result<Self, Self::Error> {
        let mut desc = "".to_string();
        if let Some(fv) = map.get(DESCRIPTION) {
            desc = fv.value.clone();
        }
        Ok(Transaction {
            tx_id: parse_field(TX_ID, map.get(TX_ID))?,
            tx_type: parse_field(TX_TYPE, map.get(TX_TYPE))?,
            from_user_id: parse_field(FROM_USER_ID, map.get(FROM_USER_ID))?,
            to_user_id: parse_field(TO_USER_ID, map.get(TO_USER_ID))?,
            amount: parse_field(AMOUNT, map.get(AMOUNT))?,
            timestamp: parse_field(TIMESTAMP, map.get(TIMESTAMP))?,
            status: parse_field(STATUS, map.get(STATUS))?,
            description: desc,
        })
    }
}

fn parse_field<T: std::str::FromStr>(field: &str, values: Option<&FieldValue>) -> Result<T, ParseError> {
    match values {
        Some(v) => {v
            .value
            .parse()
            .map_err(|_| ParseError::InvalidTextFieldValue {
                field: field.to_string(),
                value: v.value.to_string(),
                line_num: v.line,
            })}
        None => {
            Err(ParseError::FieldValueNotFound(field.to_string()))
        }
    }
}
