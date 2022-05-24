extern crate csv;
extern crate qsv_sniffer;

use std::path::Path;

use qsv_sniffer::{SampleSize, Sniffer};

fn main() {
    let data_filepath = Path::new(file!())
        .parent()
        .unwrap()
        .join("../tests/data/gdp.csv");
    let dialect = Sniffer::new()
        .sample_size(SampleSize::All)
        .sniff_path(data_filepath)
        .unwrap();
    println!("{:#?}", dialect);
}
