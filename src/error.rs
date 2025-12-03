use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Parse(String),
    Write(String),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{:?}", self)
    }
}

impl std::error::Error for Error {}
