use crate::Status;
use crate::Transaction;
use crate::TransactionType;
use crate::error::{ParseError, WriteError};
use std::io::{Read, Write};
use std::str::from_utf8;

const YPBN: [u8; 4] = [0x59, 0x50, 0x42, 0x4E];
const TX_ID_SIZE: usize = 8;
const TX_TYPE_SIZE: usize = 1;
const FROM_USER_ID_SIZE: usize = 8;
const TO_USER_ID_SIZE: usize = 8;
const AMOUNT_SIZE: usize = 8;
const TIMESTAMP_SIZE: usize = 8;
const STATUS_SIZE: usize = 1;
const DESC_LEN_SIZE: usize = 4;

const FIXED_BODY_SIZE: usize = TX_ID_SIZE
    + TX_TYPE_SIZE
    + FROM_USER_ID_SIZE
    + TO_USER_ID_SIZE
    + AMOUNT_SIZE
    + TIMESTAMP_SIZE
    + STATUS_SIZE
    + DESC_LEN_SIZE;

pub fn read<R: Read>(mut reader: R) -> Result<Vec<Transaction>, ParseError> {
    let mut transactions = Vec::new();
    let mut header = [0u8; 4];

    loop {
        match reader.read(&mut header) {
            Ok(0) => return Ok(transactions),
            Ok(_) => match header {
                YPBN => {
                    reader.read_exact(&mut header)?;
                    let size = u32::from_be_bytes(header);
                    let mut body = vec![0u8; size as usize];
                    reader.read_exact(&mut body)?;
                    transactions.push(body.try_into()?);
                }
                _ => {}
            },
            Err(e) => {
                return Err(ParseError::Io(e));
            }
        }
    }
}

pub fn write<W: Write>(w: &mut W, transactions: Vec<Transaction>) -> Result<(), WriteError> {
    for tx in transactions {
        w.write_all(&YPBN)?;
        let body: Vec<u8> = tx.into();
        let len = (body.len() as u32).to_be_bytes();
        w.write_all(&len)?;
        w.write_all(&body)?;
    }
    Ok(())
}

impl TryFrom<Vec<u8>> for Transaction {
    type Error = ParseError;

    fn try_from(body: Vec<u8>) -> Result<Self, Self::Error> {
        let (tx_id_bytes, rest) = body.split_at(TX_ID_SIZE);
        let (tx_type_bytes, rest) = rest.split_at(TX_TYPE_SIZE);
        let (from_user_bytes, rest) = rest.split_at(FROM_USER_ID_SIZE);
        let (to_user_bytes, rest) = rest.split_at(TO_USER_ID_SIZE);
        let (amount_bytes, rest) = rest.split_at(AMOUNT_SIZE);
        let (timestamp_bytes, rest) = rest.split_at(TIMESTAMP_SIZE);
        let (status_bytes, rest) = rest.split_at(STATUS_SIZE);
        let (desc_len_bytes, rest) = rest.split_at(DESC_LEN_SIZE);
        let desc_len = u32::from_be_bytes(
            desc_len_bytes
                .try_into()
                .map_err(|_| ParseError::InvalidBinaryField("description_len".to_string()))?,
        );

        Ok(Transaction {
            tx_id: parse_u64_be(tx_id_bytes, "tx_id")?,
            tx_type: tx_type_bytes
                .try_into()
                .map_err(|_| ParseError::InvalidBinaryField("tx_type".to_string()))?,
            from_user_id: parse_u64_be(from_user_bytes, "from_user_id")?,
            to_user_id: parse_u64_be(to_user_bytes, "to_user_id")?,
            amount: parse_u64_be(amount_bytes, "amount")?,
            timestamp: parse_u64_be(timestamp_bytes, "timestamp")?,
            status: status_bytes
                .try_into()
                .map_err(|_| ParseError::InvalidBinaryField("status".to_string()))?,
            description: if let Some((desc_bytes, _)) = rest.split_at_checked(desc_len as usize) {
                from_utf8(desc_bytes)
                    .map_err(|_| ParseError::InvalidBinaryField("desc_bytes".to_string()))?
                    .to_string()
            } else {
                "".to_string()
            },
        })
    }
}

fn parse_u64_be(b: &[u8], field: &str) -> Result<u64, ParseError> {
    Ok(u64::from_be_bytes(b.try_into().map_err(|_| {
        ParseError::InvalidBinaryField(field.to_string())
    })?))
}

impl From<Transaction> for Vec<u8> {
    fn from(tx: Transaction) -> Self {
        let desc = tx.description.into_bytes();
        let mut result = Vec::with_capacity(FIXED_BODY_SIZE + desc.len());

        result.extend_from_slice(&tx.tx_id.to_be_bytes());
        result.push(tx.tx_type.into());
        result.extend_from_slice(&tx.from_user_id.to_be_bytes());
        result.extend_from_slice(&tx.to_user_id.to_be_bytes());
        result.extend_from_slice(&tx.amount.to_be_bytes());
        result.extend_from_slice(&tx.timestamp.to_be_bytes());
        result.push(tx.status.into());
        result.extend_from_slice(&(desc.len() as u32).to_be_bytes());
        result.extend_from_slice(&desc);
        result
    }
}

impl TryFrom<&[u8]> for Status {
    type Error = ParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value.first() {
            None => Err(ParseError::InvalidBinaryField("status".to_string())),
            Some(b) => match *b {
                0 => Ok(Status::SUCCESS),
                1 => Ok(Status::FAILURE),
                2 => Ok(Status::PENDING),
                _ => Err(ParseError::InvalidBinaryField("status".to_string())),
            },
        }
    }
}

impl From<Status> for u8 {
    fn from(v: Status) -> Self {
        match v {
            Status::SUCCESS => 0,
            Status::FAILURE => 1,
            Status::PENDING => 2,
        }
    }
}

impl TryFrom<&[u8]> for TransactionType {
    type Error = ParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value.first() {
            None => Err(ParseError::InvalidBinaryField(
                "transaction_type".to_string(),
            )),
            Some(b) => match *b {
                0 => Ok(TransactionType::DEPOSIT),
                1 => Ok(TransactionType::TRANSFER),
                2 => Ok(TransactionType::WITHDRAWAL),
                _ => Err(ParseError::InvalidBinaryField(
                    "transaction_type".to_string(),
                )),
            },
        }
    }
}

impl From<TransactionType> for u8 {
    fn from(v: TransactionType) -> Self {
        match v {
            TransactionType::DEPOSIT => 0,
            TransactionType::TRANSFER => 1,
            TransactionType::WITHDRAWAL => 2,
        }
    }
}
