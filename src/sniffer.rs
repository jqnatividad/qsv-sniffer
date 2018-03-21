use std::collections::HashMap;
use std::path::Path;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Take};
use std::fs::File;

use regex::Regex;
use csv::{self, Reader, Terminator};
use csv_core as csvc;

use metadata::*;
use error::*;
use chain::*;
use infer_type::Types;

/// Argument used when calling `sample_size` on `Sniffer`.
#[derive(Debug, Clone)]
pub enum SampleSize {
    /// Use a number of records as the size of the sample to sniff.
    Records(usize),
    /// Use a number of bytes as the size of the sample to sniff.
    Bytes(usize),
    /// Sniff the entire sample.
    All
}

#[derive(Debug, Default)]
pub struct Sniffer {
    // CSV file dialect guesses
    delimiter: Option<u8>,
    num_preamble_rows: Option<usize>,
    has_header_row: Option<bool>,
    quote: Option<Quote>,
    flexible: Option<bool>,

    // Metadata guesses
    delimiter_freq: Option<usize>,

    // sample size to sniff
    sample_size: Option<SampleSize>,

    // state during sniffing
    max_line_length: usize,
}
impl Sniffer {
    /// Create a new CSV sniffer.
    pub fn new() -> Sniffer {
        Sniffer::default()
    }
    /// Specify the delimiter character.
    pub fn delimiter(&mut self, delimiter: u8) -> &mut Sniffer {
        self.delimiter = Some(delimiter);
        self
    }
    /// Specify the header type (whether the CSV file has a header row, and where the data starts).
    pub fn header(&mut self, header: Header) -> &mut Sniffer {
        self.num_preamble_rows = Some(header.num_preamble_rows);
        self.has_header_row = Some(header.has_header_row);
        self
    }
    // /// Specify the record terminator symbol.
    // pub fn terminator(&mut self, terminator: Terminator) -> &mut Sniffer {
    //     self.terminator = Some(terminator);
    //     self
    // }
    /// Specify the quote character (if any), and whether two quotes in a row as to be interepreted
    /// as an escaped quote.
    pub fn quote(&mut self, quote: Quote) -> &mut Sniffer {
        self.quote = Some(quote);
        self
    }

    /// The size of the sample to examine while sniffing. If using `SampleSize::Records`, the
    /// sniffer will use the value provided with `terminator()` (or `Terminator::CRLF` if no
    /// terminator is provided). Thus, `SampleSize::Records` may work unexpectedly for
    /// non-CRLF-terminated files if the terminator is not provided.
    ///
    /// The sample size defaults to `SampleSize::Bytes(4096)`.
    pub fn sample_size(&mut self, sample_size: SampleSize) -> &mut Sniffer {
        self.sample_size = Some(sample_size);
        self
    }

    pub fn open_path<P: AsRef<Path>>(&mut self, path: P) -> Result<Reader<BufReader<File>>> {
        self.open_reader(File::open(path)?)
    }
    pub fn open_reader<R: Read + Seek>(&mut self, mut reader: R) -> Result<Reader<BufReader<R>>> {
        let metadata = self.sniff_reader(&mut reader)?;
        reader.seek(SeekFrom::Start(0))?;
        metadata.dialect.open_reader(reader)
    }

    pub fn sniff_path<P: AsRef<Path>>(&mut self, path: P) -> Result<Metadata> {
        let file = File::open(path)?;
        self.sniff_reader(&file)
    }
    pub fn sniff_reader<R: Read + Seek>(&mut self, mut reader: R) -> Result<Metadata> {

        // guess quotes & delim
        self.infer_quotes_delim(&mut reader)?;

        // if we have a delimiter, we just need to search for num_preamble_rows and check for
        // flexible. Otherwise, we need to guess a delimiter as well.
        if self.delimiter.is_some() {
            self.infer_preamble_known_delim(&mut reader)?;
        } else {
            self.infer_delim_preamble(&mut reader)?;
        }

        // let (types, has_header_row) = self.infer_types(&mut reader, &quote, delim,
        //     num_preamble_rows, flex)?;

        // as this point of the process, we should have all these filled in.
        assert!(self.delimiter.is_some()
            && self.num_preamble_rows.is_some()
            && self.quote.is_some()
            && self.flexible.is_some()
            && self.delimiter_freq.is_some()
        );
        Ok(Metadata {
            dialect: Dialect {
                delimiter: self.delimiter.unwrap(),
                header: Header {
                    num_preamble_rows: self.num_preamble_rows.unwrap(),
                    has_header_row: true,
                },
                terminator: Terminator::CRLF,
                quote: self.quote.clone().unwrap(),
                doublequote_escapes: true,
                escape: Escape::Disabled,
                comment: Comment::Disabled,
                flexible: self.flexible.unwrap(),
            },
            num_fields: self.delimiter_freq.unwrap() + 1,
        })
    }

    fn take_sample_from_start<'a, R: Read + Seek>(&self, reader: &'a mut R)
        -> Result<Take<&'a mut R>>
    {
        reader.seek(SeekFrom::Start(0))?;
        self.take_sample(reader)
    }
    fn take_sample<'a, R: Read>(&self, reader: &'a mut R) -> Result<Take<&'a mut R>> {
        let sample_size = self.sample_size.clone().unwrap_or(SampleSize::Bytes(1<<14));
        let nbytes = match sample_size {
            SampleSize::Bytes(nbytes) => nbytes,
            SampleSize::Records(_) => {
                return Err(SnifferError::SniffingFailed(
                    "SampleSize::Records unimplemented".to_string()));
            },
            SampleSize::All => 1e9 as usize,
        };
        Ok(reader.take(nbytes as u64))
    }
    // Infers quotes and delimiter from quoted (or possibly quoted) files. If quotes detected,
    // updates self.quote and self.delimiter. If quotes not detected, updates self.quote to
    // Quote::None. Only valid quote characters: " (double-quote), ' (single-quote), ` (back-tick).
    fn infer_quotes_delim<R: Read + Seek>(&mut self, reader: &mut R) -> Result<()>
    {
        if let (&Some(_), &Some(_)) = (&self.quote, &self.delimiter) {
            // nothing let to infer!
            return Ok(());
        }
        let quote_guesses = match self.quote {
            Some(Quote::Some(chr)) => vec![chr],
            Some(Quote::None) => {
                // this function only checks quoted (or possibly quoted) files, nothing left to
                // do if we know there are no quotes
                return Ok(());
            },
            None => vec![b'\'', b'"', b'`']
        };
        // TODO: this can probably be replaced with a try_fold whenever that leaves nightly
        let (quote_chr, (quote_cnt, delim_guess)) = quote_guesses.iter().fold(
            Ok((b'"', (0, b'\0'))),
            |acc: Result<(u8, (usize, u8))>, &chr| {
                if let Ok(acc) = acc {
                    let mut sample_reader = self.take_sample_from_start(reader)?;
                    if let Some((cnt, delim_chr)) = quote_count(&mut sample_reader,
                        char::from(chr), &self.delimiter)?
                    {
                        Ok(if cnt > (acc.1).0 { (chr, (cnt, delim_chr)) } else { acc })
                    } else {
                        Ok(acc)
                    }
                } else {
                    acc
                }
            }
        )?;
        Ok(if quote_cnt == 0 {
            self.quote = Some(Quote::None);
        } else {
            self.quote = Some(Quote::Some(quote_chr));
            self.delimiter = Some(delim_guess);
        })
    }

    // Updates delimiter frequency, number of preamble rows, and flexible boolean.
    fn infer_preamble_known_delim<R: Read + Seek>(&mut self, reader: &mut R) -> Result<()> {
        // prerequisites for calling this function:
        assert!(self.delimiter.is_some() && self.quote.is_some());
        // unwraps for delimiter and quote are safe
        let (quote, delim) = (self.quote.clone().unwrap(), self.delimiter.unwrap());

        let sample_reader = self.take_sample_from_start(reader)?;
        let buf_reader = BufReader::new(sample_reader);

        let mut chain = Chain::default();

        if let Quote::Some(character) = quote {
            // since we have a quote, we need to run this data through the csv_core::Reader (which
            // properly escapes quoted fields
            let mut csv_reader = csvc::ReaderBuilder::new()
                .delimiter(delim)
                .quote(character).build();

            let mut output = vec![];
            let mut ends = vec![];
            for line in buf_reader.lines() {
                let line = line?;
                if line.len() > output.len() { output.resize(line.len(), 0); }
                if line.len() > ends.len() { ends.resize(line.len(), 0); }
                let (result, _, _, n_ends) = csv_reader.read_record(line.as_bytes(), &mut output,
                    &mut ends);
                // check to make sure record was read correctly
                match result {
                    csvc::ReadRecordResult::OutputFull | csvc::ReadRecordResult::OutputEndsFull => {
                        return Err(SnifferError::SniffingFailed(format!(
                            "failure to read quoted CSV record: {:?}", result)));
                    },
                    _ => {} // non-error results, do nothing
                }
                // n_ends is the number of fields, the number of delimiters would be one less than
                // this
                let freq = n_ends - 1;
                chain.add_observation(freq);
            }
        } else {
            for line in buf_reader.lines() {
                let line = line?;
                let freq = line.as_bytes().iter().filter(|&&c| c == delim).count();
                chain.add_observation(freq);
            }
        }
        self.run_chains(vec![chain])
    }

    // Updates delimiter, delimiter frequency, number of preamble rows, and flexible boolean.
    fn infer_delim_preamble<R: Read + Seek>(&mut self, reader: &mut R) -> Result<()>
    {
        let sample_reader = self.take_sample_from_start(reader)?;
        let buf_reader = BufReader::new(sample_reader);

        const NUM_ASCII_CHARS: usize = 128;
        let mut chains = vec![Chain::default(); NUM_ASCII_CHARS];
        for line in buf_reader.lines() {
            let line = line?;
            let mut freqs = [0; NUM_ASCII_CHARS];
            for &chr in line.as_bytes() {
                if chr < NUM_ASCII_CHARS as u8 {
                    freqs[chr as usize] += 1;
                }
            }
            for (chr, &freq) in freqs.iter().enumerate() {
                chains[chr as usize].add_observation(freq);
            }
        }

        self.run_chains(chains)
    }

    // Updates delimiter (if not already known), delimiter frequency, number of preamble rows, and
    // flexible boolean.
    fn run_chains(&mut self, mut chains: Vec<Chain>) -> Result<()> {
        // Find the 'best' delimiter: choose strict (non-flexible) delimiters over flexible ones,
        // and choose the one that had the highest probability markov chain in the end.
        //
        // In the case where delim is already known, 'best_delim' will be incorrect (since it won't
        // correspond with position in a vector of Chains), but we'll just ignore it when
        // constructing our return value later. 'best_state' and 'path' are necessary, though, to
        // compute the preamble rows.
        let (best_delim, delim_freq, best_state, path, _) = chains.iter_mut().enumerate()
            .fold((b',', 0, STATE_UNSTEADY, vec![], 0.0), |acc, (i, ref mut chain)| {
                let (_, _, best_state, _, best_state_prob) = acc;
                let ViterbiResults { max_delim_freq, path } = chain.viterbi();
                let (final_state, final_viter) = path[path.len() - 1];
                // println!("{} '{}' {} {:e}", i, i as u8 as char, final_state, final_viter.prob);
                // if i as u8 == b'8' {
                //     for &(state, viter) in &path {
                //         println!("{} {:e} {:?}", state, viter.prob, viter.prev);
                //     }
                // }
                if final_state < best_state
                    || (final_state == best_state && final_viter.prob > best_state_prob)
                {
                   (i as u8, max_delim_freq, final_state, path, final_viter.prob)
                } else {
                    acc
                }
            }
        );
        self.flexible = Some(match best_state {
            STATE_STEADYSTRICT => false,
            STATE_STEADYFLEX => true,
            _ => {
                return Err(SnifferError::SniffingFailed(
                    "unable to find valid delimiter".to_string()));
            }
        });

        // Find the number of preamble rows (the number of rows during which the state fluctuated
        // before getting to the final state).
        let mut num_preamble_rows = 0;
        // since path has an extra state as the beginning, skip one
        for &(state, _) in path.iter().skip(1) {
            if state == best_state {
                break;
            }
            num_preamble_rows += 1;
        }
        if self.delimiter.is_none() {
            self.delimiter = Some(best_delim);
        }
        self.delimiter_freq = Some(delim_freq);
        self.num_preamble_rows = Some(num_preamble_rows);
        Ok(())
    }

    fn infer_types<R: Read + Seek>(&mut self, reader: &mut R) -> Result<()>
    {
        let mut csv_reader = self.create_csv_reader(reader)?;
        for record in csv_reader.records() {
            let _record = record?;

        }
        Err(SnifferError::SniffingFailed("unimplemented".to_string()))
    }

    fn create_csv_reader<'a, R: Read + Seek>(&self, reader: &'a mut R)
        -> Result<Reader<BufReader<Take<&'a mut R>>>>
    {
        let sample_reader = self.take_sample_from_start(reader)?;
        let mut buf_reader = BufReader::new(sample_reader);
        if let Some(num_preamble_rows) = self.num_preamble_rows {
            for _ in 0..num_preamble_rows {
                let mut devnull = String::new();
                buf_reader.read_line(&mut devnull)?;
            }
        }

        let mut builder = csv::ReaderBuilder::new();
        if let Some(delim) = self.delimiter { builder.delimiter(delim); }
        if let Some(has_header_row) = self.has_header_row { builder.has_headers(has_header_row); }
        match self.quote {
            Some(Quote::Some(chr)) => {
                builder.quoting(true);
                builder.quote(chr);
            },
            Some(Quote::None) => {
                builder.quoting(false);
            }
            _ => {}
        }
        if let Some(flexible) = self.flexible { builder.flexible(flexible); }

        Ok(builder.from_reader(buf_reader))
    }

}


fn quote_count<R: Read>(reader: &mut R, character: char, delim: &Option<u8>)
    -> Result<Option<(usize, u8)>>
{
    let mut buf_reader = BufReader::new(reader);
    let pattern = match *delim {
        Some(delim) => format!(r#"{}\s*?{}\s*{}"#, character, delim, character),
        None => format!(r#"{}\s*?(?P<delim>[^\w\n'"`])\s*{}"#, character, character)
    };
    let re = Regex::new(&pattern).unwrap();

    // TODO: a hashmap isn't an ideal choice for this, I believe (since it requires a linear
    // search of the values at the end). Consider other options
    let mut delim_count_map: HashMap<String, usize> = HashMap::new();
    let mut count = 0;
    loop {
        let mut buf = String::new();
        let read = buf_reader.read_line(&mut buf)?;
        if read == 0 {
            break;
        }
        for cap in re.captures_iter(&mut buf) {
            count += 1;
            // if we already know delimiter, we don't need to count
            if let Some(_) = *delim {} else {
                *delim_count_map.entry(cap["delim"].to_string()).or_insert(0) += 1;
            }
        }
    }
    if count == 0 {
        return Ok(None);
    }

    // if we already know delimiter, no need to go through map
    if let Some(delim) = *delim {
        return Ok(Some((count, delim)));
    }

    // find the highest-count delimiter in the map
    let (delim_count, delim) = delim_count_map.iter().fold((0, b'\0'),
        |acc, (delim, &delim_count)| {
            assert!(delim.len() == 1);
            if delim_count > acc.0 {
                (delim_count, (delim.as_ref() as &[u8])[0])
            } else {
                acc
            }
        }
    );

    // delim_count should be nonzero; delim should always match at least something
    assert_ne!(delim_count, 0, "invalid regex match: no delimiter found");
    Ok(Some((count, delim)))
}
