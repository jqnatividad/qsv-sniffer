use std::fmt;

use crate::sniffer::DATE_PREFERENCE;
use bitflags::bitflags;
use csv::StringRecord;
use qsv_dateparser::parse_with_preference;

/// Argument used when calling `date_preference` on `Sniffer`.
#[derive(Debug, Clone, Copy)]
pub enum DatePreference {
    /// DMY format, dd/mm/yyyy, dd/mm/yy
    DmyFormat,
    /// MDY format, mm/dd/yyyy, mm/dd/yy
    MdyFormat,
}

bitflags! {
    /// Possible guesses for the field type. Implementged as a bitflag struct (see
    /// [`bitflags`](https://docs.rs/bitflags/)).
    #[derive(Default)]
    pub(crate) struct TypeGuesses: u32 {
        const BOOLEAN   = 0b00000001;
        const UNSIGNED  = 0b00000010;
        const SIGNED    = 0b00000100;
        const FLOAT     = 0b00001000;
        const DATE      = 0b00010000;
        const DATETIME  = 0b00100000;
        const TEXT      = 0b01000000;
    }
}

impl TypeGuesses {
    /// Compute the 'best-fitting' `Type` among the guesses of this struct. 'Best-fitting' in this
    /// case means the narrowest definition: `Type::Boolean` being the narrowest, and `Type::Text`
    /// being the widest (since everything can be a text field).
    pub(crate) const fn best(self) -> Type {
        // if all values are some sort of boolean (0 or 1, or 'true' and 'false'), guess boolean
        if self.contains(TypeGuesses::BOOLEAN) {
            Type::Boolean
        }
        // if all values are integer and > 0, guess unsigned
        else if self.contains(TypeGuesses::UNSIGNED) {
            Type::Unsigned
        }
        // if all values are integer, but some < 0, guess signed
        else if self.contains(TypeGuesses::SIGNED) {
            Type::Signed
        }
        // if all values are numeric, but non-integer, guess float
        else if self.contains(TypeGuesses::FLOAT) {
            Type::Float
        }
        // try datetime
        else if self.contains(TypeGuesses::DATETIME) {
            Type::DateTime
        }
        // try date
        else if self.contains(TypeGuesses::DATE) {
            Type::Date
        }
        // doesn't fit anything else, it's a text field
        else {
            Type::Text
        }
    }
    /// Returns `true` if `other` is 'allowed' in the types represented by `self`. For example,
    /// if `self` is TypesGuesses::SIGNED | TypesGuesses::FLOAT | TypeGuesses::TEXT, and `other` is
    /// TypesGuesses::TEXT, then `allows` returns `false` (since self is more restrictive than
    /// other).
    pub(crate) fn allows(self, other: &TypeGuesses) -> bool {
        !(self - *other).is_empty()
    }
}

pub(crate) fn infer_types(s: &str) -> TypeGuesses {
    if s.is_empty() {
        // empty fields can be of any type; or rather, of no known type
        return TypeGuesses::all();
    }
    let mut guesses = TypeGuesses::default();
    guesses |= TypeGuesses::TEXT;

    if s.parse::<u64>().is_ok() {
        guesses |= TypeGuesses::UNSIGNED;
    }
    if s.parse::<i64>().is_ok() {
        guesses |= TypeGuesses::SIGNED;
    }
    if s.parse::<bool>().is_ok() {
        guesses |= TypeGuesses::BOOLEAN;
    }
    if s.parse::<f64>().is_ok() {
        guesses |= TypeGuesses::FLOAT;
    }
    if let Ok(parsed_date) = parse_with_preference(
        s,
        matches!(
            DATE_PREFERENCE.with(|preference| *preference.borrow()),
            DatePreference::DmyFormat
        ),
    ) {
        // get date in rfc3339 format, if it ends with "T00:00:00+00:00"
        // its a Date type, otherwise, its DateTime.
        if parsed_date.to_rfc3339().ends_with("T00:00:00+00:00") {
            guesses |= TypeGuesses::DATE;
        } else {
            guesses |= TypeGuesses::DATETIME;
        }
    }
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
    /// Date
    Date,
    /// DateTime
    DateTime,
}
pub(crate) fn get_best_types(guesses: Vec<TypeGuesses>) -> Vec<Type> {
    guesses.iter().map(|guess| guess.best()).collect()
}
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Type::Unsigned => "Unsigned",
                Type::Signed => "Signed",
                Type::Text => "Text",
                Type::Boolean => "Boolean",
                Type::Float => "Float",
                Type::Date => "Date",
                Type::DateTime => "DateTime",
            }
        )
    }
}
