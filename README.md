# qsv CSV Sniffer

[![Documentation](https://docs.rs/qsv-sniffer/badge.svg)](https://docs.rs/qsv-sniffer)

`qsv-sniffer` provides methods to infer CSV file metadata (delimiter choice, quote character,
number of fields, field names, field data types, etc.). See the documentation for more details.

Its a detached fork of [csv-sniffer](https://github.com/jblondin/csv-sniffer) with these additional capabilities, detecting:

* utf-8 encoding
* field names
* number of rows
* additional data types (WIP)

> ℹ️ **NOTE:** This fork is optimized to support [qsv](https://github.com/jqnatividad/qsv), and its development
will be primarily dictated by qsv's requirements. Please continue to use `csv-sniffer` if you want
a general-purpose CSV sniffer.

# Setup

Add this to your `Cargo.toml`:

```toml
[dependencies]
qsv-sniffer = "0.1"
```

and this to your crate root:

```rust
crate qsv_sniffer;
```

# Example

This example shows how to write a simple command-line tool for discovering the metadata of a CSV
file:

```no_run
crate qsv_sniffer;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file>", args[0]);
        ::std::process::exit(1);
    }

    // sniff the path provided by the first argument
    match qsv_sniffer::Sniffer::new().sniff_path(&args[1]) {
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
