use super::error::Error;
use super::{Status, Transaction, TransactionType};
use std::io::{BufRead, Write};
use std::str::FromStr;

const FIELDS: &str = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION";
const FIELD_COUNT: usize = 8;

pub fn read<R: BufRead>(mut reader: R) -> Result<Vec<Transaction>, Error> {
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(_) => (),
        Err(e) => return Err(Error::Parse(format!("reading line: {:?}", e))),
    }
    if !line.trim().eq(FIELDS) {
        return Err(Error::Parse(format!("invalid headers: {:?}", line)));
    }

    let mut transactions = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(line) => match line.trim() {
                "" => continue,
                line => {
                    let tx = Transaction::try_from(line)?;
                    transactions.push(tx);
                }
            },
            Err(e) => return Err(Error::Parse(format!("reading line: {:?}", e))),
        }
    }

    Ok(transactions)
}

pub fn write<W: Write>(w: &mut W, transactions: Vec<Transaction>) -> Result<(), Error> {
    writeln!(w, "{}", FIELDS).map_err(|e| Error::Write(format!("write header: {:?}", e)))?;
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
        ).map_err(|e| Error::Write(format!("write transaction: {:?}", e)))?;
    }

    Ok(())
}

impl TryFrom<&str> for Transaction {
    type Error = Error;

    fn try_from(line: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = line.splitn(FIELD_COUNT, ',').collect();
        if parts.len() != FIELD_COUNT {
            return Err(Error::Parse(format!("field count: {:?}", parts)));
        }

        Ok(Self {
            tx_id: parts[0]
                .parse()
                .map_err(|e| Error::Parse(format!("tx_id parse: {:?}", e)))?,
            tx_type: parts[1]
                .parse()
                .map_err(|e| Error::Parse(format!("tx_type parse: {:?}", e)))?,
            from_user_id: parts[2]
                .parse()
                .map_err(|e| Error::Parse(format!("from_user_id parse: {:?}", e)))?,
            to_user_id: parts[3]
                .parse()
                .map_err(|e| Error::Parse(format!("to_user_id parse: {:?}", e)))?,
            amount: parts[4]
                .parse()
                .map_err(|e| Error::Parse(format!("amount parse: {:?}", e)))?,
            timestamp: parts[5]
                .parse()
                .map_err(|e| Error::Parse(format!("timestamp parse: {:?}", e)))?,
            status: parts[6]
                .parse()
                .map_err(|e| Error::Parse(format!("status parse: {:?}", e)))?,
            description: parts[7].to_string(),
        })
    }
}

impl FromStr for Status {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SUCCESS" => Ok(Status::SUCCESS),
            "FAILURE" => Ok(Status::FAILURE),
            "PENDING" => Ok(Status::PENDING),
            s => Err(Error::Parse(format!("unknown status: {:?}", s))),
        }
    }
}

impl FromStr for TransactionType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DEPOSIT" => Ok(TransactionType::DEPOSIT),
            "TRANSFER" => Ok(TransactionType::TRANSFER),
            "WITHDRAWAL" => Ok(TransactionType::WITHDRAWAL),
            t => Err(Error::Parse(format!("unknown transaction type: {:?}", t))),
        }
    }
}
