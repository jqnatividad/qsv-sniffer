use csv::StringRecord;

bitflags! {
    #[derive(Default)]
    pub struct TypeGuesses: u32 {
        const BOOLEAN   = 0b00000001;
        const UNSIGNED  = 0b00000010;
        const SIGNED    = 0b00000100;
        const FLOAT     = 0b00001000;
        const TEXT      = 0b00010000;
    }
}

impl TypeGuesses {
    pub fn best(&self) -> Type {
        // if all values are some sort of boolean (0 or 1, or 'true' and 'false'), guess boolean
        if      self.contains(TypeGuesses::BOOLEAN)  { Type::Boolean  }
        // if all values are integer and > 0, guess unsigned
        else if self.contains(TypeGuesses::UNSIGNED) { Type::Unsigned }
        // if all values are integer, but some < 0, guess signed
        else if self.contains(TypeGuesses::SIGNED)   { Type::Signed   }
        // if all values are numeric, but non-integer, guess float
        else if self.contains(TypeGuesses::FLOAT)    { Type::Float    }
        // doesn't fit anything else, it's a text field
        else                                         { Type::Text     }
    }
    /// Returns `true` if `other` is 'allowed' in the types represented by `self`. For example,
    /// if `self` is TypesGuesses::SIGNED | TypesGuesses::FLOAT | TypeGuesses::TEXT, and `other` is
    /// TypesGuesses::TEXT, then `allows` returns `false` (since self is more restrictive than
    /// other).
    pub fn allows(&self, other: &TypeGuesses) -> bool {
        !(*self - *other).is_empty()
    }
}

pub fn infer_types(s: &str) -> TypeGuesses {
    if s.len() == 0 {
        // empty fields can be of any type; or rather, of no known type
        return TypeGuesses::all();
    }
    let mut guesses = TypeGuesses::default();
    guesses |= TypeGuesses::TEXT;
    if let Ok(_) = s.parse::<u64>() { guesses |= TypeGuesses::UNSIGNED; }
    if let Ok(_) = s.parse::<i64>() { guesses |= TypeGuesses::SIGNED; }
    if let Ok(_) = s.parse::<bool>() { guesses |= TypeGuesses::BOOLEAN; }
    if let Ok(_) = s.parse::<f64>() { guesses |= TypeGuesses::FLOAT; }
    guesses
}

pub fn infer_record_types(record: &StringRecord) -> Vec<TypeGuesses> {
    record.iter().map(|s| infer_types(s)).collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Unsigned,
    Signed,
    Text,
    Boolean,
    Float,
}
pub fn get_best_types(guesses: Vec<TypeGuesses>) -> Vec<Type> {
    guesses.iter().map(|guess| guess.best()).collect()
}
