// Copyright 2023 electronfraud
//
// This file is part of calc.
//
// Foobar is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// Foobar is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along with
// Foobar. If not, see <https://www.gnu.org/licenses/>.

//! Arithmetic with units.

use super::{Error, Unit};

/// A number with an optional unit.
#[derive(Clone, Debug)]
pub struct Number {
    pub value: f64,
    pub unit: Option<Unit>,
}

impl Number {
    /// Returns a dimensionless `Number`.
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self { value, unit: None }
    }

    /// Returns a `Number` with the same value as this one but different units.
    /// No unit conversion is performed.
    #[must_use]
    pub const fn with_unit(&self, unit: Unit) -> Number {
        Number {
            value: self.value,
            unit: Some(unit),
        }
    }

    /// Returns true if this number has no units.
    #[must_use]
    pub fn is_dimensionless(&self) -> bool {
        self.unit.is_none()
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let value = if self.value < 0.001 || self.value >= 10_000_000_000.0 {
            let e = format!("{:.6e}", self.value);
            let halves: Vec<&str> = e.splitn(2, 'e').collect();
            halves[0]
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
                + "e"
                + halves[1]
        } else {
            format!("{:.6}", self.value)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        };

        #[allow(clippy::map_unwrap_or)] // can't because of `f` borrow
        self.unit
            .as_ref()
            .map(|u| write!(f, "[{value} {u}]"))
            .unwrap_or_else(|| write!(f, "{value}"))
    }
}

impl std::ops::Add<&Number> for &Number {
    type Output = Result<Number, Error>;

    /// Adds this number to another number.
    ///
    /// # Errors
    ///
    /// Returns an error if `self` and `other` have incommensurable units.
    fn add(self, other: &Number) -> Result<Number, Error> {
        let v1 = self.value;
        let v2 = other.value;

        match (&self.unit, &other.unit) {
            (Some(u1), Some(u2)) => u2
                .convert_interval(v2, u1)
                .map(|v2| Number::new(v1 + v2).with_unit(u1.clone())),
            (None, None) => Ok(Number::new(v1 + v2)),
            (Some(u1), None) => Err(Error::IncommensurableUnits(
                Some(Box::new(u1.clone())),
                None,
            )),
            (None, Some(u2)) => Err(Error::IncommensurableUnits(
                None,
                Some(Box::new(u2.clone())),
            )),
        }
    }
}

impl std::ops::Sub<&Number> for &Number {
    type Output = Result<Number, Error>;

    /// Subtracts a number from this number.
    ///
    /// # Errors
    ///
    /// Returns an error if `self` and `other` have incommensurable units.
    fn sub(self, other: &Number) -> Result<Number, Error> {
        let v1 = self.value;
        let v2 = other.value;

        match (&self.unit, &other.unit) {
            (Some(u1), Some(u2)) => u2
                .convert_interval(v2, u1)
                .map(|v2| Number::new(v1 - v2).with_unit(u1.clone())),
            (None, None) => Ok(Number::new(v1 - v2)),
            (Some(u1), None) => Err(Error::IncommensurableUnits(
                Some(Box::new(u1.clone())),
                None,
            )),
            (None, Some(u2)) => Err(Error::IncommensurableUnits(
                None,
                Some(Box::new(u2.clone())),
            )),
        }
    }
}

impl std::ops::Mul<&Number> for &Number {
    type Output = Number;

    /// Multiplies this number by another.
    fn mul(self, other: &Number) -> Number {
        let v1 = self.value;
        let v2 = other.value;

        match (&self.unit, &other.unit) {
            (Some(u1), Some(u2)) => Number::new(v1 * v2).with_unit(u1 * u2),
            (Some(u), None) | (None, Some(u)) => Number::new(v1 * v2).with_unit(u.clone()),
            (None, None) => Number::new(v1 * v2),
        }
    }
}

impl std::ops::Mul<&Unit> for &Number {
    type Output = Number;

    /// Multiplies this number's unit by another unit. If the number has no
    /// unit, assigns the unit to the number.
    fn mul(self, other: &Unit) -> Number {
        self.with_unit(self.unit.as_ref().map_or(other.clone(), |u| u * other))
    }
}

impl std::ops::Div<&Number> for &Number {
    type Output = Number;

    /// Divides this number by another.
    fn div(self, other: &Number) -> Number {
        let v1 = self.value;
        let v2 = other.value;

        match (&self.unit, &other.unit) {
            (Some(u1), Some(u2)) => Number::new(v1 / v2).with_unit(u1 / u2),
            (Some(u1), None) => Number::new(v1 / v2).with_unit(u1.clone()),
            (None, Some(u2)) => Number::new(v1 / v2).with_unit(u2.inverse()),
            (None, None) => Number::new(v1 / v2),
        }
    }
}

impl std::ops::Div<&Unit> for &Number {
    type Output = Number;

    /// Divides this number's unit by another unit. If the number has no unit,
    /// assigns the inverse of the unit to the number.
    fn div(self, other: &Unit) -> Number {
        self.with_unit(self.unit.as_ref().map_or(other.inverse(), |u| u / other))
    }
}

#[cfg(test)]
mod tests {
    use crate::units::Number;
    use crate::units::{HOUR, KILOGRAM, METER, MILE, SECOND};

    #[test]
    fn dimensionless_added_to_dimensionless() {
        let x = (&Number::new(5.0) + &Number::new(10.0)).unwrap();
        assert_eq!(x.value, 15.0);
        assert!(x.is_dimensionless());
    }

    #[test]
    fn dimensionless_added_to_unit() {
        let result = &Number::new(5.0) + &Number::new(10.0).with_unit(&METER / &SECOND);
        assert!(result.is_err());
    }

    #[test]
    fn unit_added_to_dimensionless() {
        let result = &Number::new(10.0).with_unit(&METER / &SECOND) + &Number::new(5.0);
        assert!(result.is_err());
    }

    #[test]
    fn unit_added_to_compatible_unit() {
        let x = (&Number::new(10.0).with_unit(&METER / &SECOND)
            + &Number::new(5.0).with_unit(&MILE / &HOUR))
            .unwrap();
        assert_eq!(x.value, 12.235199999999999);
        assert_eq!(x.unit.as_ref().unwrap().numer, vec![&METER]);
        assert_eq!(x.unit.unwrap().denom, vec![&SECOND]);
    }

    #[test]
    fn unit_added_to_incompatible_unit() {
        let result = &Number::new(10.0).with_unit(&METER / &SECOND)
            + &Number::new(5.0).with_unit(&MILE / &KILOGRAM);
        assert!(result.is_err());
    }

    #[test]
    fn dimensionless_multiplied_by_dimensionless() {
        let x = &Number::new(5.0) * &Number::new(10.0);
        assert_eq!(x.value, 50.0);
        assert!(x.is_dimensionless());
    }

    #[test]
    fn dimensionless_multiplied_by_unit() {
        let x = &Number::new(5.0) * &Number::new(10.0).with_unit(&METER / &SECOND);
        assert_eq!(x.value, 50.0);
        assert_eq!(x.unit.as_ref().unwrap().numer, vec![&METER]);
        assert_eq!(x.unit.unwrap().denom, vec![&SECOND]);
    }

    #[test]
    fn unit_multiplied_by_dimensionless() {
        let x = &Number::new(5.0).with_unit(&METER / &SECOND) * &Number::new(10.0);
        assert_eq!(x.value, 50.0);
        assert_eq!(x.unit.as_ref().unwrap().numer, vec![&METER]);
        assert_eq!(x.unit.unwrap().denom, vec![&SECOND]);
    }

    #[test]
    fn unit_multiplied_by_unit() {
        let x = &Number::new(5.0).with_unit(&METER / &SECOND)
            * &Number::new(10.0).with_unit(&MILE / &HOUR);
        assert_eq!(x.value, 50.0);
        assert_eq!(x.unit.as_ref().unwrap().numer, vec![&METER, &MILE]);
        assert_eq!(x.unit.unwrap().denom, vec![&SECOND, &HOUR]);
    }
}
