extern crate csv;
extern crate csv_sniffer;

use std::path::Path;

use csv::Terminator;
use csv_sniffer::metadata::*;
use csv_sniffer::{SampleSize, Sniffer, Type};

#[test]
fn test_semicolon() {
    let data_filepath = Path::new(file!())
        .parent()
        .unwrap()
        .join("data/2016_presidential_election_durham.csv");
    let metadata = Sniffer::new()
        .sample_size(SampleSize::All)
        .sniff_path(data_filepath)
        .unwrap();
    assert_eq!(
        metadata,
        Metadata {
            dialect: Dialect {
                delimiter: b';',
                header: Header {
                    has_header_row: true,
                    num_preamble_rows: 0,
                },
                terminator: Terminator::CRLF,
                quote: Quote::None,
                doublequote_escapes: true,
                escape: Escape::Disabled,
                comment: Comment::Disabled,
                flexible: false
            },
            num_fields: 5,
            types: vec![
                Type::Text,
                Type::Text,
                Type::Unsigned,
                Type::Text,
                Type::Text
            ]
        }
    );
}

#[test]
fn test_comma() {
    let data_filepath = Path::new(file!())
        .parent()
        .unwrap()
        .join("data/library-visitors.csv");
    let metadata = Sniffer::new()
        .sample_size(SampleSize::All)
        .sniff_path(data_filepath)
        .unwrap();
    assert_eq!(
        metadata,
        Metadata {
            dialect: Dialect {
                delimiter: b',',
                header: Header {
                    has_header_row: true,
                    num_preamble_rows: 0,
                },
                terminator: Terminator::CRLF,
                quote: Quote::None,
                doublequote_escapes: true,
                escape: Escape::Disabled,
                comment: Comment::Disabled,
                flexible: false
            },
            num_fields: 5,
            types: vec![
                Type::Text,
                Type::Unsigned,
                Type::Unsigned,
                Type::Unsigned,
                Type::Unsigned
            ]
        }
    );
}

#[test]
fn test_flexible() {
    let data_filepath = Path::new(file!())
        .parent()
        .unwrap()
        .join("data/gotriangle-routes-cary-ch-duke-durham-raleigh-wofline.csv");
    let metadata = Sniffer::new()
        .sample_size(SampleSize::All)
        .sniff_path(data_filepath)
        .unwrap();
    assert_eq!(
        metadata,
        Metadata {
            dialect: Dialect {
                delimiter: b',',
                header: Header {
                    has_header_row: true,
                    num_preamble_rows: 0,
                },
                terminator: Terminator::CRLF,
                quote: Quote::None,
                doublequote_escapes: true,
                escape: Escape::Disabled,
                comment: Comment::Disabled,
                flexible: true
            },
            num_fields: 7,
            types: vec![
                Type::Text,
                Type::Unsigned,
                Type::Unsigned,
                Type::Text,
                Type::Text,
                Type::Unsigned,
                Type::Text
            ]
        }
    );
}
