use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid headers, expected {expected}, found {found}")]
    InvalidHeaders { expected: String, found: String },

    #[error("invalid fields count {fields_count} in line {line_num}")]
    InvalidFieldsCount {
        line_num: usize,
        fields_count: usize,
    },

    #[error("invalid {field} value {value} in line {line_num} ")]
    InvalidTextFieldValue {
        field: String,
        value: String,
        line_num: usize,
    },

    #[error("field {0} value not found")]
    FieldValueNotFound(String),

    #[error("invalid field {0}")]
    InvalidBinaryField(String),
}

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
