/*!
Error types and conversions for the `csv-sniffer` crate.
*/
use std::error::Error;
use std::io;
use std::fmt;

use csv;

/// An error that occurs while examining a CSV data file.
#[derive(Debug)]
pub enum SnifferError {
    /// An I/O error
    Io(io::Error),
    /// A CSV parsing error (from the csv crate)
    Csv(csv::Error),
    /// A CSV sniffing error
    SniffingFailed(String),
}

/// Ease-of-use `Result` type with a `SnifferError`.
pub type Result<T> = ::std::result::Result<T, SnifferError>;

impl fmt::Display for SnifferError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SnifferError::Io(ref err) => write!(f, "IO error: {}", err),
            SnifferError::Csv(ref err) => write!(f, "CSV read error: {}", err),
            SnifferError::SniffingFailed(ref s) => write!(f, "Sniffing failed: {}", s),
        }
    }
}

impl Error for SnifferError {
    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            SnifferError::Io(ref err) => Some(err),
            SnifferError::Csv(ref err) => Some(err),
            SnifferError::SniffingFailed(_) => None,
        }
    }
}

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
