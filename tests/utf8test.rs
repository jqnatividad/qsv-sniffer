extern crate csv;
extern crate csv_sniffer;

use std::path::Path;

use csv_sniffer::metadata::*;
use csv_sniffer::{SampleSize, Sniffer, Type};

#[test]
fn test_utf8() {
    let data_filepath = Path::new(file!())
        .parent()
        .unwrap()
        .join("data/test-utf8.csv");
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
                    num_preamble_rows: 0
                },
                quote: Quote::None,
                flexible: false,
                is_utf8: false
            },
            num_fields: 11,
            types: vec![
                Type::Text,
                Type::Unsigned,
                Type::Float,
                Type::Float,
                Type::Text,
                Type::Text,
                Type::Text,
                Type::Text,
                Type::Text,
                Type::Text,
                Type::Text
            ]
        }
    );
}
