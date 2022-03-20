/*!
This `csv-sniffer` crate provides methods to infer CSV file details (delimiter choice, quote
character, number of fields, field data types, etc.).

# Overview

The [`Sniffer`](struct.Sniffer.html) type is the primary entry point for using this crate. Its
[`Sniffer::open_path`](struct.Sniffer.html#method.open_path) and
[`Sniffer::open_reader`](struct.Sniffer.html#method.open_reader) methods return a configured
[`csv::Reader`](https://docs.rs/csv).

Alternatively, the [`Sniffer::sniff_path`](struct.Sniffer.html#method.sniff_path) and
[`Sniffer::sniff_reader`](struct.Sniffer.html#method.sniff_reader) methods return a
[`Metadata`](metadata/struct.Metadata.html) object containing the deduced details about the
underlying CSV input.

This sniffer detects the following metadata about a CSV file:

* Delimiter -- byte character between fields in a record
* Number of preamble rows -- number of rows in a CSV file before the data starts (occasionally used
in data files to introduce the data)
* Has a header row? -- whether or not the first row of the data file provdes column headers
* Quote -- byte character (either ", ', or `) used to quote fields, or that the file has no quotes
* Flexible -- whether or not records are all of the same length
* Delimiter count -- maximum number of delimiters in each row (and therefore number of fields in
each row)
* Types -- the inferred data type of each field in the data table

See [`Metadata`](metadata/struct.Metadata.html) for full information about what the sniffer returns.

# Setup

Add this to your `Cargo.toml`:

```toml
[dependencies]
csv-sniffer = "0.1"
```

and this to your crate root:

```rust
extern crate csv_sniffer;
```

# Example

This example shows how to write a simple command-line tool for discovering the metadata of a CSV
file:

```no_run
extern crate csv_sniffer;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file>", args[0]);
        ::std::process::exit(1);
    }

    // sniff the path provided by the first argument
    match csv_sniffer::Sniffer::new().sniff_path(&args[1]) {
        Ok(metadata) => {
            println!("{}", metadata);
        },
        Err(err) => {
            eprintln!("ERROR: {}", err);
        }
    }
}
```

This example is provided as the primary binary for this crate. In the source directory, this can be
run as:

```ignore
$ cargo run -- tests/data/library-visitors.csv
```

*/

#![warn(missing_docs)]

extern crate csv;
extern crate csv_core;
extern crate regex;
#[macro_use]
extern crate bitflags;
extern crate memchr;

pub(crate) mod chain;
pub mod error;
pub mod metadata;

mod sniffer;
pub use sniffer::Sniffer;

mod sample;
pub use sample::SampleSize;

pub(crate) mod field_type;
pub use field_type::Type;

mod snip;
