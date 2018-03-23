# CSV Sniffer

[![Build Status](https://travis-ci.org/jblondin/csv-sniffer.svg?branch=master)](https://travis-ci.org/jblondin/csv-sniffer)
[![Documentation](https://docs.rs/csv-sniffer/badge.svg)](https://docs.rs/csv-sniffer)

This `csv-sniffer` crate provides methods to infer CSV file details (delimiter choice, quote
character, number of fields, field data types, etc.). See the documentation for more details.

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
