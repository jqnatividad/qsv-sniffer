
bitflags! {
    #[derive(Default)]
    pub struct Types: u32 {
        const UNSIGNED  = 0b00000001;
        const SIGNED    = 0b00000010;
        const TEXT      = 0b00000100;
        const BOOLEAN   = 0b00001000;
        const FLOAT     = 0b00010000;
    }
}

pub fn infer_types(s: &str) -> Types {
    let mut types = Types::default();
    types |= Types::TEXT;
    if let Ok(_) = s.parse::<u64>() { types |= Types::UNSIGNED; }
    if let Ok(_) = s.parse::<i64>() { types |= Types::SIGNED; }
    if let Ok(_) = s.parse::<bool>() { types |= Types::BOOLEAN; }
    if let Ok(_) = s.parse::<f64>() { types |= Types::FLOAT; }
    types
}
