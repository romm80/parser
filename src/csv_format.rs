use super::error::{ParseError, WriteError};
use super::{Status, Transaction, TransactionType};
use std::io::{BufRead, Write};
use std::str::FromStr;

const HEADERS: &str = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION";
const HEADERS_COUNT: usize = 8;

pub fn read<R: BufRead>(mut reader: R) -> Result<Vec<Transaction>, ParseError> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    if !line.trim().eq(HEADERS) {
        return Err(ParseError::InvalidHeaders {
            expected: HEADERS.to_string(),
            found: line,
        });
    }

    let mut transactions = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line_num = idx + 2;
        match line {
            Ok(line) => match line.trim() {
                "" => continue,
                line => transactions.push(Transaction::try_from((line, line_num))?),
            },
            Err(e) => return Err(ParseError::Io(e)),
        }
    }

    Ok(transactions)
}

pub fn write<W: Write>(w: &mut W, transactions: Vec<Transaction>) -> Result<(), WriteError> {
    writeln!(w, "{}", HEADERS)?;
    for tx in transactions {
        writeln!(
            w,
            "{},{},{},{},{},{},{},{}",
            tx.tx_id,
            tx.tx_type,
            tx.from_user_id,
            tx.to_user_id,
            tx.amount,
            tx.timestamp,
            tx.status,
            tx.description,
        )?;
    }

    Ok(())
}

impl TryFrom<(&str, usize)> for Transaction {
    type Error = ParseError;

    fn try_from((line, line_num): (&str, usize)) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = line.splitn(HEADERS_COUNT, ',').collect();
        Ok(Self {
            tx_id: parse_field("TX_ID", parts.get(0), line_num)?,
            tx_type: parse_field("TX_TYPE", parts.get(1), line_num)?,
            from_user_id: parse_field("FROM_USER_ID", parts.get(2), line_num)?,
            to_user_id: parse_field("TO_USER_ID", parts.get(3), line_num)?,
            amount: parse_field("AMOUNT", parts.get(4), line_num)?,
            timestamp: parse_field("TIMESTAMP", parts.get(5), line_num)?,
            status: parse_field("STATUS", parts.get(6), line_num)?,
            description: parts.get(7).unwrap_or(&"").to_string(),
        })
    }
}

fn parse_field<T: FromStr>(
    field: &str,
    value: Option<&&str>,
    line_num: usize,
) -> Result<T, ParseError> {
    match value {
        Some(v) => v.parse().map_err(|_| ParseError::InvalidTextFieldValue {
            field: field.to_string(),
            value: v.to_string(),
            line_num,
        }),
        None => Err(ParseError::FieldValueNotFound(field.to_string())),
    }
}

impl FromStr for Status {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SUCCESS" => Ok(Status::SUCCESS),
            "FAILURE" => Ok(Status::FAILURE),
            "PENDING" => Ok(Status::PENDING),
            _ => Err(()),
        }
    }
}

impl FromStr for TransactionType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DEPOSIT" => Ok(TransactionType::DEPOSIT),
            "TRANSFER" => Ok(TransactionType::TRANSFER),
            "WITHDRAWAL" => Ok(TransactionType::WITHDRAWAL),
            _ => Err(()),
        }
    }
}
