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
