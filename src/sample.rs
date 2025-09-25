use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

use crate::error::Result;
use crate::sniffer::IS_UTF8;

/// Argument used when calling `sample_size` on `Sniffer`.
#[derive(Debug, Clone, Copy)]
pub enum SampleSize {
    /// Use a number of records as the size of the sample to sniff.
    Records(usize),
    /// Use a number of bytes as the size of the sample to sniff.
    Bytes(usize),
    /// Sniff the entire sample.
    All,
}

pub fn take_sample_from_start<R>(
    reader: &mut R,
    sample_size: SampleSize,
) -> Result<SampleIter<'_, R>>
where
    R: Read + Seek,
{
    reader.seek(SeekFrom::Start(0))?;
    Ok(SampleIter::new(reader, sample_size))
}

pub struct SampleIter<'a, R: 'a + Read> {
    reader: BufReader<&'a mut R>,
    sample_size: SampleSize,
    n_bytes: usize,
    n_records: usize,
    is_done: bool,
}

impl<'a, R: Read> SampleIter<'a, R> {
    fn new(reader: &'a mut R, sample_size: SampleSize) -> SampleIter<'a, R> {
        let buf_reader = BufReader::new(reader);
        SampleIter {
            reader: buf_reader,
            sample_size,
            n_bytes: 0,
            n_records: 0,
            is_done: false,
        }
    }
}

impl<'a, R: Read> Iterator for SampleIter<'a, R> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Result<String>> {
        if self.is_done {
            return None;
        }

        let mut buf = Vec::new();
        let n_bytes_read = match self.reader.read_until(b'\n', &mut buf) {
            Ok(n_bytes_read) => n_bytes_read,
            Err(e) => {
                return Some(Err(e.into()));
            }
        };
        if n_bytes_read == 0 {
            self.is_done = true;
            return None;
        }

        let mut output = simdutf8::basic::from_utf8(&buf).map_or_else(
            |_| {
                // Its not all utf-8, set IS_UTF8 global to false
                IS_UTF8.with(|flag| {
                    *flag.borrow_mut() = false;
                });
                String::from_utf8_lossy(&buf).to_string()
            },
            |str_utf8| str_utf8.to_string(),
        );

        let last_byte = (output.as_ref() as &[u8])[output.len() - 1];
        if last_byte != b'\n' && last_byte != b'\r' {
            // non CR/LF-ended line
            // line was cut off before ending, so we ignore it!
            self.is_done = true;
            return None;
        }

        output = output.trim_matches(|c| c == '\n' || c == '\r').into();
        self.n_bytes += n_bytes_read;
        self.n_records += 1;
        match self.sample_size {
            SampleSize::Records(max_records) => {
                if self.n_records > max_records {
                    self.is_done = true;
                    return None;
                }
            }
            SampleSize::Bytes(max_bytes) => {
                if self.n_bytes > max_bytes {
                    self.is_done = true;
                    return None;
                }
            }
            SampleSize::All => {}
        }
        Some(Ok(output))
    }
}
