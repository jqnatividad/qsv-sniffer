extern crate csv;
extern crate csv_sniffer;

use std::path::Path;

use csv::Terminator;
use csv_sniffer::metadata::*;

fn main() {
    let data_filepath = Path::new(file!())
        .parent()
        .unwrap()
        .join("../tests/data/gdp.csv");
    let dialect = Dialect {
        delimiter: b',',
        header: Header {
            has_header_row: true,
            num_preamble_rows: 4,
        },
        quote: Quote::Some(b'"'),
        doublequote_escapes: true,
        comment: Comment::Disabled,
        escape: Escape::Disabled,
        terminator: Terminator::CRLF,
        flexible: false,
    };
    let mut reader = dialect.open_path(data_filepath).unwrap();
    for result in reader.records() {
        let record = result.unwrap();
        println!("{:?}", record);
        break;
    }
}
