extern crate csv;
extern crate csv_core;
extern crate regex;
#[macro_use] extern crate bitflags;

pub mod metadata;
pub use metadata::{Metadata, Dialect};

pub mod error;

pub(crate) mod chain;

mod sniffer;
pub use sniffer::{Sniffer, SampleSize};

mod infer_type;
