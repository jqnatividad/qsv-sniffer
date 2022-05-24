extern crate csv;
extern crate qsv_sniffer;

use std::path::Path;

use qsv_sniffer::metadata::*;

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
        flexible: false,
        is_utf8: true,
    };
    let mut reader = dialect.open_path(data_filepath).unwrap();
    for result in reader.records() {
        let record = result.unwrap();
        println!("{:?}", record);
        break;
    }
}
