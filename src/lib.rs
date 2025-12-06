mod bin_format;
mod csv_format;
mod error;
mod txt_format;

use crate::error::ParseError;
use error::WriteError;
use std::fmt::Display;
use std::io::{BufReader, BufWriter, Read, Write};

pub const TX_ID: &str = "TX_ID";
pub const TX_TYPE: &str = "TX_TYPE";
pub const FROM_USER_ID: &str = "FROM_USER_ID";
pub const TO_USER_ID: &str = "TO_USER_ID";
pub const AMOUNT: &str = "AMOUNT";
pub const TIMESTAMP: &str = "TIMESTAMP";
pub const STATUS: &str = "STATUS";
pub const DESCRIPTION: &str = "DESCRIPTION";

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

pub fn read<R: Read>(data: R, format: Format) -> Result<Vec<Transaction>, ParseError> {
    let reader = BufReader::new(data);
    match format {
        Format::Csv => csv_format::read(reader),
        Format::Bin => bin_format::read(reader),
        Format::Txt => txt_format::read(reader),
    }
}

pub fn write<W: Write>(data: W, format: Format, tx: Vec<Transaction>) -> Result<(), WriteError> {
    let mut buf = BufWriter::new(data);
    match format {
        Format::Csv => csv_format::write(&mut buf, tx)?,
        Format::Bin => bin_format::write(&mut buf, tx)?,
        Format::Txt => txt_format::write(&mut buf, tx)?,
    }
    buf.flush().map_err(|e| WriteError::Io(e))
}

#[cfg(test)]
mod tests_csv {
    use super::*;
    use std::io::Cursor;

    fn transactions() -> Vec<Transaction> {
        vec![
            Transaction {
                tx_id: 1,
                tx_type: TransactionType::DEPOSIT,
                from_user_id: 12,
                to_user_id: 13,
                amount: 14,
                timestamp: 15,
                status: Status::SUCCESS,
                description: r#"desc, "desc1""#.to_string(),
            },
            Transaction {
                tx_id: 2,
                tx_type: TransactionType::WITHDRAWAL,
                from_user_id: 22,
                to_user_id: 33,
                amount: 44,
                timestamp: 55,
                status: Status::FAILURE,
                description: r#""desc2""#.to_string(),
            },
        ]
    }

    #[test]
    fn test_read_succeed() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,DEPOSIT,12,13,14,15,SUCCESS,desc, "desc1"
2,WITHDRAWAL,22,33,44,55,FAILURE,"desc2""#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Csv).unwrap();
        assert_eq!(tx, transactions())
    }

    #[test]
    fn test_write_succeed() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,DEPOSIT,12,13,14,15,SUCCESS,desc, "desc1"
2,WITHDRAWAL,22,33,44,55,FAILURE,"desc2"
"#;

        let mut output = Vec::new();
        let _ = write(&mut output, Format::Csv, transactions()).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert_eq!(data, result)
    }

    #[test]
    fn test_read_headers_failed() {
        let data = r#"TX,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,DEPOSIT,12,13,14,15,SUCCESS,desc, "desc1""#;

        let input = Cursor::new(data.as_bytes());
        let err = read(input, Format::Csv).unwrap_err();
        assert!(matches!(err, ParseError::InvalidHeaders { .. }));
    }

    #[test]
    fn test_read_field_not_found() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,DEPOSIT"#;

        let input = Cursor::new(data.as_bytes());
        let err = read(input, Format::Csv).unwrap_err();
        assert!(matches!(err, ParseError::FieldValueNotFound(_),));
    }

    #[test]
    fn test_read_tx_id_failed() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
x,DEPOSIT,12,13,14,15,SUCCESS,desc, "desc1""#;

        let input = Cursor::new(data.as_bytes());
        let err = read(input, Format::Csv).unwrap_err();
        assert!(matches!(err, ParseError::InvalidTextFieldValue { .. },));
    }

    #[test]
    fn test_read_tx_type_failed() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,dep,12,13,14,15,SUCCESS,desc, "desc1""#;

        let input = Cursor::new(data.as_bytes());
        let err = read(input, Format::Csv).unwrap_err();
        assert!(matches!(err, ParseError::InvalidTextFieldValue { .. },));
    }

    #[test]
    fn test_read_user_from_failed() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,DEPOSIT,x,13,14,15,SUCCESS,desc, "desc1""#;

        let input = Cursor::new(data.as_bytes());
        let err = read(input, Format::Csv).unwrap_err();
        assert!(matches!(err, ParseError::InvalidTextFieldValue { .. },));
    }

    #[test]
    fn test_read_user_to_failed() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,DEPOSIT,12,x,14,15,SUCCESS,desc, "desc1""#;

        let input = Cursor::new(data.as_bytes());
        let err = read(input, Format::Csv).unwrap_err();
        assert!(matches!(err, ParseError::InvalidTextFieldValue { .. },));
    }

    #[test]
    fn test_read_amount_failed() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,DEPOSIT,12,13,x,15,SUCCESS,desc, "desc1""#;

        let input = Cursor::new(data.as_bytes());
        let err = read(input, Format::Csv).unwrap_err();
        assert!(matches!(err, ParseError::InvalidTextFieldValue { .. },));
    }

    #[test]
    fn test_read_timestamp_failed() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,DEPOSIT,12,13,14,x,SUCCESS,desc, "desc1""#;

        let input = Cursor::new(data.as_bytes());
        let err = read(input, Format::Csv).unwrap_err();
        assert!(matches!(err, ParseError::InvalidTextFieldValue { .. },));
    }

    #[test]
    fn test_read_status_failed() {
        let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1,DEPOSIT,12,13,14,15,succ,desc, "desc1""#;

        let input = Cursor::new(data.as_bytes());
        let err = read(input, Format::Csv).unwrap_err();
        assert!(matches!(err, ParseError::InvalidTextFieldValue { .. },));
    }
}

#[cfg(test)]
mod tests_txt {
    use super::*;
    use std::io::Cursor;

    fn transactions() -> Vec<Transaction> {
        vec![
            Transaction {
                tx_id: 1,
                tx_type: TransactionType::DEPOSIT,
                from_user_id: 2,
                to_user_id: 3,
                amount: 4,
                timestamp: 5,
                status: Status::SUCCESS,
                description: r#"desc, "desc1""#.to_string(),
            },
            Transaction {
                tx_id: 11,
                tx_type: TransactionType::WITHDRAWAL,
                from_user_id: 22,
                to_user_id: 33,
                amount: 44,
                timestamp: 55,
                status: Status::FAILURE,
                description: "".to_string(),
            },
        ]
    }

    #[test]
    fn test_read_succeed() {
        let data = r#"
# Record 1 (DEPOSIT)
TX_TYPE: DEPOSIT
TO_USER_ID: 3
FROM_USER_ID: 2
TIMESTAMP: 5
DESCRIPTION: desc, "desc1"
TX_ID: 1
AMOUNT: 4
STATUS: SUCCESS

TX_TYPE: WITHDRAWAL
TO_USER_ID: 33
FROM_USER_ID: 22
TIMESTAMP: 55
TX_ID: 11
AMOUNT: 44
STATUS: FAILURE
"#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Txt).unwrap();
        assert_eq!(tx, transactions())
    }

    #[test]
    fn test_write_succeed() {
        let data = concat!(
            r#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 2
TO_USER_ID: 3
AMOUNT: 4
TIMESTAMP: 5
STATUS: SUCCESS
DESCRIPTION: desc, "desc1"

TX_ID: 11
TX_TYPE: WITHDRAWAL
FROM_USER_ID: 22
TO_USER_ID: 33
AMOUNT: 44
TIMESTAMP: 55
STATUS: FAILURE
DESCRIPTION: "#,
            "\n\n"
        );

        let mut output = Vec::new();
        let _ = write(&mut output, Format::Txt, transactions()).unwrap();
        let result = String::from_utf8(output).unwrap();
        assert_eq!(data, result);
    }

    #[test]
    fn test_read_tx_id_failed() {
        let data = r#"
# Record 1 (DEPOSIT)
TX_TYPE: DEPOSIT
TO_USER_ID: 3
FROM_USER_ID: 2
TIMESTAMP: 5
DESCRIPTION: desc, "desc1"
TX_ID: x
AMOUNT: 4
STATUS: SUCCESS
"#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Txt).unwrap_err();
        assert!(matches!(tx, ParseError::InvalidTextFieldValue { .. },));
    }

    #[test]
    fn test_read_tx_id_not_found() {
        let data = r#"
# Record 1 (DEPOSIT)
TX_TYPE: DEPOSIT
TO_USER_ID: 3
FROM_USER_ID: 2
TIMESTAMP: 5
DESCRIPTION: desc, "desc1"
AMOUNT: 4
STATUS: SUCCESS
"#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Txt).unwrap_err();
        assert!(matches!(tx, ParseError::FieldValueNotFound { .. },));
    }

    #[test]
    fn test_read_tx_type_failed() {
        let data = r#"
# Record 1 (DEPOSIT)
TX_TYPE: dep
TO_USER_ID: 3
FROM_USER_ID: 2
TIMESTAMP: 5
DESCRIPTION: desc, "desc1"
TX_ID: 1
AMOUNT: 4
STATUS: SUCCESS
"#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Txt).unwrap_err();
        assert!(matches!(tx, ParseError::InvalidTextFieldValue { .. },));
    }

    #[test]
    fn test_read_tx_type_not_found() {
        let data = r#"
# Record 1 (DEPOSIT)
TO_USER_ID: 3
FROM_USER_ID: 2
TIMESTAMP: 5
DESCRIPTION: desc, "desc1"
AMOUNT: 4
TX_ID: 1
STATUS: SUCCESS
"#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Txt).unwrap_err();
        assert!(matches!(tx, ParseError::FieldValueNotFound { .. },));
    }

    #[test]
    fn test_read_from_user_failed() {
        let data = r#"
# Record 1 (DEPOSIT)
TX_TYPE: DEPOSIT
TO_USER_ID: 3
FROM_USER_ID: x
TIMESTAMP: 5
DESCRIPTION: desc, "desc1"
TX_ID: 1
AMOUNT: 4
STATUS: SUCCESS
"#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Txt).unwrap_err();
        assert!(matches!(tx, ParseError::InvalidTextFieldValue { .. },));
    }

    #[test]
    fn test_read_to_user_not_found() {
        let data = r#"
# Record 1 (DEPOSIT)
TX_TYPE: DEPOSIT
TX_ID: 1
TO_USER_ID: 3
TIMESTAMP: 5
DESCRIPTION: desc, "desc1"
AMOUNT: 4
STATUS: SUCCESS
"#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Txt).unwrap_err();
        assert!(matches!(tx, ParseError::FieldValueNotFound { .. },));
    }

    #[test]
    fn test_read_amount_failed() {
        let data = r#"
# Record 1 (DEPOSIT)
TX_TYPE: DEPOSIT
TO_USER_ID: 3
FROM_USER_ID: 2
TIMESTAMP: 5
DESCRIPTION: desc, "desc1"
TX_ID: 1
AMOUNT: x
STATUS: SUCCESS
"#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Txt).unwrap_err();
        assert!(matches!(tx, ParseError::InvalidTextFieldValue { .. },));
    }

    #[test]
    fn test_read_amount_not_found() {
        let data = r#"
# Record 1 (DEPOSIT)
TX_TYPE: DEPOSIT
TO_USER_ID: 3
FROM_USER_ID: 2
TIMESTAMP: 5
DESCRIPTION: desc, "desc1"
STATUS: SUCCESS
TX_ID: 1
"#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Txt).unwrap_err();
        assert!(matches!(tx, ParseError::FieldValueNotFound { .. },));
    }

    #[test]
    fn test_read_timestamp_failed() {
        let data = r#"
# Record 1 (DEPOSIT)
TX_TYPE: DEPOSIT
TO_USER_ID: 3
FROM_USER_ID: 2
TIMESTAMP: x
DESCRIPTION: desc, "desc1"
TX_ID: 1
AMOUNT: 4
STATUS: SUCCESS
"#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Txt).unwrap_err();
        assert!(matches!(tx, ParseError::InvalidTextFieldValue { .. },));
    }

    #[test]
    fn test_read_timestamp_not_found() {
        let data = r#"
# Record 1 (DEPOSIT)
TX_TYPE: DEPOSIT
TO_USER_ID: 3
FROM_USER_ID: 2
DESCRIPTION: desc, "desc1"
AMOUNT: 4
TX_ID: 1
STATUS: SUCCESS
"#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Txt).unwrap_err();
        assert!(matches!(tx, ParseError::FieldValueNotFound { .. },));
    }

    #[test]
    fn test_read_status_failed() {
        let data = r#"
# Record 1 (DEPOSIT)
TX_TYPE: DEPOSIT
TO_USER_ID: 3
FROM_USER_ID: 2
TIMESTAMP: 5
DESCRIPTION: desc, "desc1"
TX_ID: 1
AMOUNT: 4
STATUS: succ
"#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Txt).unwrap_err();
        assert!(matches!(tx, ParseError::InvalidTextFieldValue { .. },));
    }

    #[test]
    fn test_read_status_not_found() {
        let data = r#"
# Record 1 (DEPOSIT)
TX_TYPE: DEPOSIT
TO_USER_ID: 3
FROM_USER_ID: 2
DESCRIPTION: desc, "desc1"
AMOUNT: 4
TIMESTAMP: 5
TX_ID: 1
"#;

        let input = Cursor::new(data.as_bytes());
        let tx = read(input, Format::Txt).unwrap_err();
        assert!(matches!(tx, ParseError::FieldValueNotFound { .. },));
    }
}

#[cfg(test)]
mod tests_bin {
    use std::io::Cursor;
    use super::*;

    fn transactions() -> Vec<Transaction> {
        vec![
            Transaction {
                tx_id: 1,
                tx_type: TransactionType::DEPOSIT,
                from_user_id: 2,
                to_user_id: 3,
                amount: 4,
                timestamp: 5,
                status: Status::SUCCESS,
                description: r#"desc, "desc1""#.to_string(),
            },
            Transaction {
                tx_id: 11,
                tx_type: TransactionType::WITHDRAWAL,
                from_user_id: 22,
                to_user_id: 33,
                amount: 44,
                timestamp: 55,
                status: Status::FAILURE,
                description: "".to_string(),
            },
        ]
    }

    #[test]
    fn test_write_read_succeed() {
        let expect = transactions();

        let mut output = Vec::new();
        write(&mut output, Format::Bin, transactions()).unwrap();

        let input = Cursor::new(output);
        let result = read(input, Format::Bin).unwrap();

        assert_eq!(result, expect);
    }
}