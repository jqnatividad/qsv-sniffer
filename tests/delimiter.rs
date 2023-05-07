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
            avg_record_len: 35,
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
            avg_record_len: 30,
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
fn test_boolean() {
    let data_filepath = Path::new(file!())
        .parent()
        .unwrap()
        .join("data/library-visitors-boolean.csv");
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
            avg_record_len: 31,
            num_fields: 6,
            fields: vec![
                "Month".to_string(),
                "Door Count".to_string(),
                "Web Site Visits".to_string(),
                "Catalog Visits".to_string(),
                "Overdrive Visits".to_string(),
                "Vacation".to_string()
            ],
            types: vec![
                Type::Text,
                Type::Unsigned,
                Type::Unsigned,
                Type::Unsigned,
                Type::Unsigned,
                Type::Boolean
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
            avg_record_len: 112,
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

#[test]
fn test_utf8_again() {
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
            avg_record_len: 137,
            num_fields: 11,
            fields: vec![
                "DIA.DESEMB".to_string(),
                "COD.SUBITEM.NCM".to_string(),
                "VMLE.DOLAR.BAL.EXP".to_string(),
                "PESO.LIQ.MERC.BAL.EXP".to_string(),
                "COD.IMPDR.EXPDR".to_string(),
                "NOME.IMPDR.EXPDR".to_string(),
                "PAIS.ORIGEM.DESTINO".to_string(),
                "UA.LOCAL.DESBQ.EMBQ".to_string(),
                "NOME.IMPORTADOR.ESTRANGEIRO".to_string(),
                "NUM.DDE".to_string(),
                "NUM.RE".to_string()
            ],
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

#[test]
fn test_date_sniffing() {
    let data_filepath = Path::new(file!())
        .parent()
        .unwrap()
        .join("data/boston311.csv");
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
                    num_preamble_rows: 0
                },
                quote: Quote::None,
                flexible: false,
                is_utf8: true
            },
            avg_record_len: 433,
            num_fields: 29,
            fields: vec![
                "case_enquiry_id".to_string(),
                "open_dt".to_string(),
                "target_dt".to_string(),
                "closed_dt".to_string(),
                "ontime".to_string(),
                "case_status".to_string(),
                "closure_reason".to_string(),
                "case_title".to_string(),
                "subject".to_string(),
                "reason".to_string(),
                "type".to_string(),
                "queue".to_string(),
                "department".to_string(),
                "submittedphoto".to_string(),
                "closedphoto".to_string(),
                "location".to_string(),
                "fire_district".to_string(),
                "pwd_district".to_string(),
                "city_council_district".to_string(),
                "police_district".to_string(),
                "neighborhood".to_string(),
                "neighborhood_services_district".to_string(),
                "ward".to_string(),
                "precinct".to_string(),
                "location_street_name".to_string(),
                "location_zipcode".to_string(),
                "latitude".to_string(),
                "longitude".to_string(),
                "source".to_string(),
            ],
            types: vec![
                Type::Unsigned,
                Type::DateTime,
                Type::DateTime,
                Type::DateTime,
                Type::Text,
                Type::Text,
                Type::Text,
                Type::Text,
                Type::Text,
                Type::Text,
                Type::Text,
                Type::Text,
                Type::Text,
                Type::Text,
                Type::NULL,
                Type::Text,
                Type::Unsigned,
                Type::Text,
                Type::Unsigned,
                Type::Text,
                Type::Text,
                Type::Unsigned,
                Type::Text,
                Type::Unsigned,
                Type::Text,
                Type::Unsigned,
                Type::Float,
                Type::Float,
                Type::Text
            ]
        }
    );
}
