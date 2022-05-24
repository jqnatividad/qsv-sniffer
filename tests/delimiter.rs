extern crate csv;
extern crate qsv_sniffer;

use std::path::Path;

use qsv_sniffer::metadata::*;
use qsv_sniffer::{SampleSize, Sniffer, Type};

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
                quote: Quote::None,
                flexible: false,
                is_utf8: true
            },
            num_fields: 5,
            fields: vec![
                "Name".to_string(),
                "Party".to_string(),
                "Vote Count".to_string(),
                "Voting Method".to_string(),
                "Voting Preinct".to_string()
            ],
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
                quote: Quote::None,
                flexible: false,
                is_utf8: true,
            },
            num_fields: 5,
            fields: vec![
                "Month".to_string(),
                "Door Count".to_string(),
                "Web Site Visits".to_string(),
                "Catalog Visits".to_string(),
                "Overdrive Visits".to_string()
            ],
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
                quote: Quote::None,
                flexible: true,
                is_utf8: true,
            },
            num_fields: 7,
            fields: vec![
                "municipality".to_string(),
                "agency_id".to_string(),
                "route_id".to_string(),
                "route_short_name".to_string(),
                "route_long_name".to_string(),
                "min_headway_minutes".to_string(),
                "route_url".to_string()
            ],
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
