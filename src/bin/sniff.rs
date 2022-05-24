extern crate qsv_sniffer;

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
        }
        Err(err) => {
            eprintln!("ERROR: {}", err);
        }
    }
}
