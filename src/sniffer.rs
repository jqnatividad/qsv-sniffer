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
    // hints about CSV file
    delimiter: Option<u8>,
    header: Option<Header>,
    // terminator: Option<Terminator>,
    quote: Option<Quote>,
    escape: Option<Escape>,
    comment: Option<Comment>,

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
        self.header = Some(header);
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
    /// Specify the escape character.
    pub fn escape(&mut self, escape: Escape) -> &mut Sniffer {
        self.escape = Some(escape);
        self
    }
    /// Specify the comment character.
    pub fn comment(&mut self, comment: Comment) -> &mut Sniffer {
        self.comment = Some(comment);
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

    pub fn open_path<P: AsRef<Path>>(&self, path: P) -> Result<Reader<BufReader<File>>> {
        self.open_reader(File::open(path)?)
    }
    pub fn open_reader<R: Read + Seek>(&self, mut reader: R) -> Result<Reader<BufReader<R>>> {
        let metadata = self.sniff_reader(&mut reader)?;
        reader.seek(SeekFrom::Start(0))?;
        metadata.dialect.open_reader(reader)
    }

    pub fn sniff_path<P: AsRef<Path>>(&self, path: P) -> Result<Metadata> {
        let file = File::open(path)?;
        self.sniff_reader(&file)
    }
    pub fn sniff_reader<R: Read + Seek>(&self, mut reader: R) -> Result<Metadata> {

        // guess quotes & delim
        let (quote, delim) = self.infer_quotes_delim(&mut reader, &self.quote, &self.delimiter)?;

        // if quote is Quote::None, delim was not guessed
        let (delim, delim_freq, num_preamble_rows, flex) = if let Quote::None = quote {
            // check to see if we have a delimiter provided
            match (self.delimiter, &self.header) {
                (Some(delimiter), &Some(Header { num_preamble_rows, .. })) => {
                    let (delim_freq, _, flex) =
                        self.infer_preamble_known_delim(&mut reader, &quote, delim)?;
                    (delimiter, delim_freq, num_preamble_rows, flex)
                },
                (Some(delimiter), &None) => {
                    let (delim_freq, num_preamble_rows, flex) =
                        self.infer_preamble_known_delim(&mut reader, &quote, delim)?;
                    (delimiter, delim_freq, num_preamble_rows, flex)
                },
                (None, _) => {
                    // ok, we really don't have a delimiter, time to look for one
                    self.infer_delim_preamble(&mut reader)?
                }
            }
        } else {
            let (delim_freq, num_preamble_rows, flex) =
                self.infer_preamble_known_delim(&mut reader, &quote, delim)?;
            (delim, delim_freq, num_preamble_rows, flex)
        };

        // let (types, has_header_row) = self.infer_types(&mut reader, &quote, delim,
        //     num_preamble_rows, flex)?;

        let header = self.header.clone().unwrap_or(
            Header { num_preamble_rows: num_preamble_rows, has_header_row: true });
        let escape = self.escape.clone().unwrap_or(Escape::Disabled);
        let comment = self.comment.clone().unwrap_or(Comment::Disabled);

        Ok(Metadata {
            dialect: Dialect {
                delimiter: delim,
                header: header,
                terminator: Terminator::CRLF,
                quote: quote,
                escape: escape,
                comment: comment,
                flexible: flex,
            },
            num_fields: delim_freq + 1,
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
                // If terminator isn't set, default use CRLF
                // self.terminator = self.terminator.or(Some(Terminator::CRLF));
                return Err(SnifferError::SniffingFailed(
                    "SampleSize::Records unimplemented".to_string()));
            },
            SampleSize::All => 1e9 as usize,
        };
        Ok(reader.take(nbytes as u64))
    }
    // this will either return Quote::Some {..} and the guessed delimiter, or Quote::None and an
    // invalid delimiter
    fn infer_quotes_delim<R: Read + Seek>(&self, reader: &mut R, quote: &Option<Quote>,
        delim: &Option<u8>) -> Result<(Quote, u8)>
    {
        match (quote, delim) {
            (&Some(ref quote), &Some(ref delim)) => { return Ok((quote.clone(), *delim)); },
            _ => {}
        }
        let quote_guesses = match *quote {
            Some(Quote::Some { character: chr, .. }) => vec![chr],
            Some(Quote::None) => { return Ok((Quote::None, b'\0')); },
            None => vec![b'\'', b'"', b'`']
        };
        // TODO: this can probably be replaced with a try_fold whenever that leaves nightly
        let (quote_chr, (quote_cnt, delim_guess)) = quote_guesses.iter().fold(
            Ok((b'"', (0, b'\0'))),
            |acc: Result<(u8, (usize, u8))>, &chr| {
                if let Ok(acc) = acc {
                    let mut sample_reader = self.take_sample_from_start(reader)?;
                    if let Some((cnt, delim_chr)) = quote_count(&mut sample_reader,
                        char::from(chr), delim)?
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
            (Quote::None, b'\0')
        } else {
            (
                Quote::Some {
                    character: quote_chr,
                    doublequote_escapes: true
                },
                delim_guess
            )
        })
    }

    // Returns result with (delimiter frequency, number of preamble rows, flexible or not)
    fn infer_preamble_known_delim<R: Read + Seek>(&self, reader: &mut R, quote: &Quote, delim: u8)
        -> Result<(usize, usize, bool)>
    {
        let sample_reader = self.take_sample_from_start(reader)?;
        let buf_reader = BufReader::new(sample_reader);

        let mut chain = Chain::default();

        if let &Quote::Some { character, .. } = quote {
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
        let (_, delim_freq, num_preamble_rows, flex) = self.run_chains(vec![chain])?;

        Ok((delim_freq, num_preamble_rows, flex))
    }

    // Returns result with (delimiter, delimiter frequency, number of preamble rows, flexible)
    fn infer_delim_preamble<R: Read + Seek>(&self, reader: &mut R)
        -> Result<(u8, usize, usize, bool)>
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

        Ok(self.run_chains(chains)?)
    }

    // Returns result with (delimiter, delimiter frequency, number of preamble rows, flexible)
    fn run_chains(&self, mut chains: Vec<Chain>) -> Result<(u8, usize, usize, bool)> {
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
        if best_state == STATE_UNSTEADY {
            return Err(SnifferError::SniffingFailed("unable to find valid delimiter".to_string()));
        }
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
        Ok((best_delim, delim_freq, num_preamble_rows, best_state == STATE_STEADYFLEX))
    }

    fn infer_types<R: Read + Seek>(&self, reader: &mut R, quote: &Quote, delim: u8,
        num_preamble_rows: usize, flexible: bool) -> Result<(Vec<Types>, bool)>
    {
        let sample_reader = self.take_sample_from_start(reader)?;
        let mut buf_reader = BufReader::new(sample_reader);
        for _ in 0..num_preamble_rows {
            let mut devnull = String::new();
            buf_reader.read_line(&mut devnull)?;
        }

        let mut bldr = csv::ReaderBuilder::new();
        bldr.delimiter(delim)
            .flexible(flexible);
        match *quote {
            Quote::None => {
                bldr.quoting(false);
            }
            Quote::Some { character, doublequote_escapes } => {
                bldr.double_quote(doublequote_escapes)
                    .quoting(true)
                    .quote(character);
            }
        }
        let mut csv_reader = bldr.from_reader(buf_reader);
        for record in csv_reader.records() {
            let _record = record?;

        }
        Err(SnifferError::SniffingFailed("unimplemented".to_string()))
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


