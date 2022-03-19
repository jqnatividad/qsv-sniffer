use std::fmt;

use csv::StringRecord;

bitflags! {
    /// Possible guesses for the field type. Implementged as a bitflag struct (see
    /// [`bitflags`](https://docs.rs/bitflags/)).
    #[derive(Default)]
    pub(crate) struct TypeGuesses: u32 {
        const BOOLEAN   = 0b00000001;
        const UNSIGNED  = 0b00000010;
        const SIGNED    = 0b00000100;
        const FLOAT     = 0b00001000;
        const TEXT      = 0b00010000;
    }
}

impl TypeGuesses {
    /// Compute the 'best-fitting' `Type` among the guesses of this struct. 'Best-fitting' in this
    /// case means the narrowest definition: `Type::Boolean` being the narrowest, and `Type::Text`
    /// being the widest (since everything can be a text field).
    pub(crate) fn best(&self) -> Type {
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
    pub(crate) fn allows(&self, other: &TypeGuesses) -> bool {
        !(*self - *other).is_empty()
    }
}

pub(crate) fn infer_types(s: &str) -> TypeGuesses {
    if s.is_empty() {
        // empty fields can be of any type; or rather, of no known type
        return TypeGuesses::all();
    }
    let mut guesses = TypeGuesses::default();
    guesses |= TypeGuesses::TEXT;
    if s.parse::<u64>().is_ok() { guesses |= TypeGuesses::UNSIGNED; }
    if s.parse::<i64>().is_ok() { guesses |= TypeGuesses::SIGNED; }
    if s.parse::<bool>().is_ok() { guesses |= TypeGuesses::BOOLEAN; }
    if s.parse::<f64>().is_ok() { guesses |= TypeGuesses::FLOAT; }
    guesses
}

pub(crate) fn infer_record_types(record: &StringRecord) -> Vec<TypeGuesses> {
    record.iter().map(infer_types).collect()
}

/// The valid field types for fields in a CSV record.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    /// Unsigned integer (integer >= 0)
    Unsigned,
    /// Signed integer
    Signed,
    /// Text (any field can be a type)
    Text,
    /// Boolean (true / false or 0 / 1)
    Boolean,
    /// Floating-point
    Float,
}
pub(crate) fn get_best_types(guesses: Vec<TypeGuesses>) -> Vec<Type> {
    guesses.iter().map(|guess| guess.best()).collect()
}
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Type::Unsigned => "Unsigned",
            Type::Signed   => "Signed",
            Type::Text     => "Text",
            Type::Boolean  => "Boolean",
            Type::Float    => "Float",
        })
    }
}
