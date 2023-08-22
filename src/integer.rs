//! Integers with specified representations.

use crate::units;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Representation {
    Binary,
    Decimal,
    Octal,
    Hexadecimal,
}

/// An integer represented in hexadecimal, octal, or binary.
#[derive(Clone, Debug, PartialEq)]
pub struct Integer {
    pub value: i64,
    pub repr: Representation,
}

impl Integer {
    /// Converts a string slice in hexadecimal, decimal, octal, or binary.
    ///
    /// Recognized prefixes for non-decimal numbers are:
    /// - `0x`, `0X`, `$` (hexadecimal)
    /// - `0o`, `0O`, `0` (octal)
    /// - `0b`, `0B` (binary)
    ///
    /// You can add underscores to non-decimal numbers, and commas to decimal
    /// numbers, to make them more readable.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use calc::integer::{Integer, Representation::*};
    /// assert_eq!(Integer::parse("0xcafe"), Some(Integer { value: 0xcafe, repr: Hexadecimal }));
    /// assert_eq!(Integer::parse("123,456,789"), Some(Integer { value: 123456789, repr: Decimal }));
    /// assert_eq!(Integer::parse("0774"), Some(Integer { value: 0o774, repr: Octal }));
    /// assert_eq!(Integer::parse("0b110100_11101101"), Some(Integer { value: 0b11010011101101, repr: Binary }));
    /// ```
    #[must_use]
    pub fn parse(s: &str) -> Option<Integer> {
        if s.starts_with("0x") || s.starts_with("0X") {
            let s = s.replace('_', "");
            if let Ok(value) = i64::from_str_radix(&s[2..s.len()], 16) {
                return Some(Integer::hex(value));
            }
        } else if s.starts_with('$') {
            let s = s.replace('_', "");
            if let Ok(value) = i64::from_str_radix(&s[1..s.len()], 16) {
                return Some(Integer::hex(value));
            }
        } else if s.starts_with("0b") || s.starts_with("0B") {
            let s = s.replace('_', "");
            if let Ok(value) = i64::from_str_radix(&s[2..s.len()], 2) {
                return Some(Integer::bin(value));
            }
        } else if s.starts_with("0o") || s.starts_with("0O") {
            let s = s.replace('_', "");
            if let Ok(value) = i64::from_str_radix(&s[2..s.len()], 8) {
                return Some(Integer::oct(value));
            }
        } else if s.starts_with('0') {
            let s = s.replace('_', "");
            if let Ok(value) = i64::from_str_radix(&s[1..s.len()], 8) {
                return Some(Integer::oct(value));
            }
        } else {
            let s = s.replace(',', "");
            if let Ok(value) = &s.parse::<i64>() {
                return Some(Integer::dec(*value));
            }
        }
        None
    }

    /// Make a new integer.
    #[must_use]
    pub fn new(value: i64, repr: Representation) -> Integer {
        Integer { value, repr }
    }

    /// Make a new integer with binary representation.
    #[must_use]
    pub fn bin(value: i64) -> Integer {
        Integer {
            value,
            repr: Representation::Binary,
        }
    }

    /// Make a new integer with decimal representation.
    #[must_use]
    pub fn dec(value: i64) -> Integer {
        Integer {
            value,
            repr: Representation::Decimal,
        }
    }

    /// Make a new integer with octal representation.
    #[must_use]
    pub fn oct(value: i64) -> Integer {
        Integer {
            value,
            repr: Representation::Octal,
        }
    }

    /// Make a new integer with hexadecimal representation.
    #[must_use]
    pub fn hex(value: i64) -> Integer {
        Integer {
            value,
            repr: Representation::Hexadecimal,
        }
    }

    /// Make a new integer with the same value but a different representation.
    #[must_use]
    pub fn with_repr(&self, repr: Representation) -> Integer {
        Integer {
            value: self.value,
            repr,
        }
    }

    /// Converts this integer into a dimensionless floating-point number.
    #[must_use]
    pub fn as_units_number(&self) -> units::Number {
        // TODO: Return an Err when self.value won't fit into an f64
        #[allow(clippy::cast_precision_loss)]
        units::Number::new(self.value as f64)
    }
}

/// Adds separators to a string.
///
/// Starting from the end, `sep` is inserted every `part_len` characters,
/// ignoring `prefix_len` characters at the beginning of the string.
fn separators(s: String, sep: char, part_len: usize, prefix_len: usize) -> String {
    let mut s = s;
    let mut ix = s.len();
    while ix > part_len + prefix_len {
        ix -= part_len;
        s.insert(ix, sep);
    }
    s
}

impl std::fmt::Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.repr {
            Representation::Binary => {
                f.write_str(&separators(format!("0b{:b}", self.value), '_', 8, 2))
            }
            Representation::Decimal => {
                f.write_str(&separators(format!("{:?}", self.value), ',', 3, 0))
            }
            Representation::Octal => {
                f.write_str(&separators(format!("0{:o}", self.value), '_', 3, 1))
            }
            Representation::Hexadecimal => {
                f.write_str(&separators(format!("0x{:x}", self.value), '_', 8, 2))
            }
        }
    }
}

impl std::ops::Add<&Integer> for &Integer {
    type Output = Integer;

    fn add(self, other: &Integer) -> Integer {
        Integer::new(self.value + other.value, self.repr)
    }
}

impl std::ops::Sub<&Integer> for &Integer {
    type Output = Integer;

    fn sub(self, other: &Integer) -> Integer {
        Integer::new(self.value - other.value, self.repr)
    }
}

impl std::ops::Mul<&Integer> for &Integer {
    type Output = Integer;

    fn mul(self, other: &Integer) -> Integer {
        Integer::new(self.value * other.value, self.repr)
    }
}

impl std::ops::Div<&Integer> for &Integer {
    type Output = units::Number;

    fn div(self, other: &Integer) -> units::Number {
        // TODO: Return an Err when self.value won't fit into an f64
        #[allow(clippy::cast_precision_loss)]
        units::Number::new((self.value as f64) / (other.value as f64))
    }
}

impl std::ops::Add<&units::Number> for &Integer {
    type Output = Result<units::Number, units::Error>;

    fn add(self, other: &units::Number) -> Result<units::Number, units::Error> {
        &self.as_units_number() + other
    }
}

impl std::ops::Sub<&units::Number> for &Integer {
    type Output = Result<units::Number, units::Error>;

    fn sub(self, other: &units::Number) -> Result<units::Number, units::Error> {
        &self.as_units_number() - other
    }
}

impl std::ops::Mul<&units::Number> for &Integer {
    type Output = Result<units::Number, units::Error>;

    fn mul(self, other: &units::Number) -> Result<units::Number, units::Error> {
        &self.as_units_number() * other
    }
}

impl std::ops::Div<&units::Number> for &Integer {
    type Output = Result<units::Number, units::Error>;

    fn div(self, other: &units::Number) -> Result<units::Number, units::Error> {
        &self.as_units_number() / other
    }
}

impl std::ops::Mul<&units::Unit> for &Integer {
    type Output = Result<units::Number, units::Error>;

    fn mul(self, other: &units::Unit) -> Result<units::Number, units::Error> {
        &self.as_units_number() * other
    }
}

impl std::ops::Div<&units::Unit> for &Integer {
    type Output = Result<units::Number, units::Error>;

    fn div(self, other: &units::Unit) -> Result<units::Number, units::Error> {
        &self.as_units_number() / other
    }
}

#[cfg(test)]
mod tests {
    use crate::integer::Integer;

    #[test]
    fn bin_display() {
        let b = Integer::bin(0b101011001100010101101);
        assert_eq!(b.to_string(), "0b10101_10011000_10101101");
    }

    #[test]
    fn oct_display() {
        let b = Integer::oct(0o72625173);
        assert_eq!(b.to_string(), "072_625_173");
    }

    #[test]
    fn dec_display() {
        let b = Integer::dec(12345678);
        assert_eq!(b.to_string(), "12,345,678");
    }

    #[test]
    fn hex_display() {
        let b = Integer::hex(0xbeefcafeface);
        assert_eq!(b.to_string(), "0xbeef_cafeface");
    }
}
