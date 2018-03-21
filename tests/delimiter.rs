extern crate csv;
extern crate csv_sniffer;

use std::path::Path;

use csv_sniffer::{Sniffer, SampleSize};
use csv_sniffer::metadata::*;
use csv::Terminator;

#[test]
fn test_semicolon() {
    let data_filepath = Path::new(file!()).parent().unwrap().join(
        "data/2016_presidential_election_durham.csv");
    let metadata = Sniffer::new().sample_size(SampleSize::All).sniff_path(data_filepath).unwrap();
    assert_eq!(metadata, Metadata {
        dialect: Dialect {
            delimiter: b';',
            header: Header {
                has_header_row: true,
                num_preamble_rows: 0,
            },
            terminator: Terminator::CRLF,
            quote: Quote::None,
            escape: Escape::Disabled,
            comment: Comment::Disabled,
            flexible: false
        },
        num_fields: 5
    });
}

#[test]
fn test_comma() {
    let data_filepath = Path::new(file!()).parent().unwrap().join("data/library-visitors.csv");
    let metadata = Sniffer::new().sample_size(SampleSize::All).sniff_path(data_filepath).unwrap();
    assert_eq!(metadata, Metadata {
        dialect: Dialect {
            delimiter: b',',
            header: Header {
                has_header_row: true,
                num_preamble_rows: 0,
            },
            terminator: Terminator::CRLF,
            quote: Quote::None,
            escape: Escape::Disabled,
            comment: Comment::Disabled,
            flexible: false
        },
        num_fields: 5
    });
}

#[test]
fn test_flexible() {
    let data_filepath = Path::new(file!()).parent().unwrap().join(
        "data/gotriangle-routes-cary-ch-duke-durham-raleigh-wofline.csv");
    let metadata = Sniffer::new().sample_size(SampleSize::All).sniff_path(data_filepath).unwrap();
    assert_eq!(metadata, Metadata {
        dialect: Dialect {
            delimiter: b',',
            header: Header {
                has_header_row: true,
                num_preamble_rows: 0,
            },
            terminator: Terminator::CRLF,
            quote: Quote::None,
            escape: Escape::Disabled,
            comment: Comment::Disabled,
            flexible: true
        },
        num_fields: 7
    });
}
