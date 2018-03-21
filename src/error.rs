use std::io;
use csv;

/// An error that occurs while examining a CSV data file.
#[derive(Debug)]
pub enum SnifferError {
    /// An I/O error
    Io(io::Error),
    /// A CSV parsing error (from the csv crate)
    Csv(csv::Error),
    SniffingFailed(String),
}

pub type Result<T> = ::std::result::Result<T, SnifferError>;

impl From<io::Error> for SnifferError {
    fn from(err: io::Error) -> SnifferError {
        SnifferError::Io(err)
    }
}
impl From<csv::Error> for SnifferError {
    fn from(err: csv::Error) -> SnifferError {
        SnifferError::Csv(err)
    }
}
