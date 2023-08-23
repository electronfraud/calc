// Copyright 2023 electronfraud
//
// This file is part of calc.
//
// calc is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// calc is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details.
//
// You should have received a copy of the GNU General Public License along with
// calc. If not, see <https://www.gnu.org/licenses/>.

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

/// Helper for `std::fmt::Display` implementation.
fn should_use_exponent_format(x: f64) -> bool {
    // I don't know if these thresholds make sense, or if thresholds are even
    // the right way to deal with formatting choices. In casual use these seem
    // to be ok though.
    x.is_finite() && x != 0.0 && (x.abs() < 0.001 || x.abs() >= 10_000_000_000.0)
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        // Use exponent format for very small and very large numbers. Use
        // decimal format for everything else (including NaNs and infinites).
        let value = if self.value == -0.0 {
            "0".to_string()
        } else if should_use_exponent_format(self.value) {
            // Use exponent format, but trim trailing zeroes. Then, delete the
            // decimal point if the entire fractional component was zeroes.
            let e = format!("{:.6e}", self.value);
            let halves: Vec<&str> = e.splitn(2, 'e').collect();
            halves[0]
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
                + "e"
                + halves[1]
        } else {
            // Use decimal format, but trim trailing zeroes. Then, delete the
            // decimal point if the entire fractional component was zeroes.
            format!("{:.6}", self.value)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        };

        // Add the number's unit, if it has one.
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
                .convert(v2, u1)
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
                .convert(v2, u1)
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
    type Output = Result<Number, Error>;

    /// Multiplies this number by another.
    fn mul(self, other: &Number) -> Result<Number, Error> {
        let v1 = self.value;
        let v2 = other.value;

        match (&self.unit, &other.unit) {
            (Some(u1), Some(u2)) => (u1 * u2).map(|u| Number::new(v1 * v2).with_unit(u)),
            (Some(u), None) | (None, Some(u)) => Ok(Number::new(v1 * v2).with_unit(u.clone())),
            (None, None) => Ok(Number::new(v1 * v2)),
        }
    }
}

impl std::ops::Mul<&Unit> for &Number {
    type Output = Result<Number, Error>;

    /// Multiplies this number's unit by another unit. If the number has no
    /// unit, assigns the unit to the number.
    fn mul(self, other: &Unit) -> Result<Number, Error> {
        self.unit
            .as_ref()
            .map_or(Ok(other.clone()), |u| u * other)
            .map(|u| self.with_unit(u))
    }
}

impl std::ops::Div<&Number> for &Number {
    type Output = Result<Number, Error>;

    /// Divides this number by another.
    fn div(self, other: &Number) -> Result<Number, Error> {
        let v1 = self.value;
        let v2 = other.value;

        match (&self.unit, &other.unit) {
            (Some(u1), Some(u2)) => (u1 / u2).map(|u| Number::new(v1 / v2).with_unit(u)),
            (Some(u1), None) => Ok(Number::new(v1 / v2).with_unit(u1.clone())),
            (None, Some(u2)) => u2.inverse().map(|u| Number::new(v1 / v2).with_unit(u)),
            (None, None) => Ok(Number::new(v1 / v2)),
        }
    }
}

impl std::ops::Div<&Unit> for &Number {
    type Output = Result<Number, Error>;

    /// Divides this number's unit by another unit. If the number has no unit,
    /// assigns the inverse of the unit to the number.
    fn div(self, other: &Unit) -> Result<Number, Error> {
        self.unit
            .as_ref()
            .map_or(other.inverse(), |u| u / other)
            .map(|u| self.with_unit(u))
    }
}

#[cfg(test)]
mod tests {
    use crate::units::Number;
    use crate::units::{HOUR, KILOGRAM, METER, MILE, SECOND, TEMP_CELSIUS};

    #[test]
    fn dimensionless_added_to_dimensionless() {
        let x = (&Number::new(5.0) + &Number::new(10.0)).unwrap();
        assert_eq!(x.value, 15.0);
        assert!(x.is_dimensionless());
    }

    #[test]
    fn dimensionless_added_to_number_with_unit() {
        let result = &Number::new(5.0) + &Number::new(10.0).with_unit((&METER / &SECOND).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn number_with_unit_added_to_dimensionless() {
        let result = &Number::new(10.0).with_unit((&METER / &SECOND).unwrap()) + &Number::new(5.0);
        assert!(result.is_err());
    }

    #[test]
    fn number_with_unit_added_to_compatible_number_with_unit() {
        let x = (&Number::new(10.0).with_unit((&METER / &SECOND).unwrap())
            + &Number::new(5.0).with_unit((&MILE / &HOUR).unwrap()))
            .unwrap();
        assert_eq!(x.value, 12.235199999999999);
        assert_eq!(*x.unit.as_ref().unwrap().numer(), vec![&METER]);
        assert_eq!(*x.unit.unwrap().denom(), vec![&SECOND]);
    }

    #[test]
    fn number_with_unit_added_to_incompatible_number_with_unit() {
        let result = &Number::new(10.0).with_unit((&METER / &SECOND).unwrap())
            + &Number::new(5.0).with_unit((&MILE / &KILOGRAM).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn dimensionless_subtracted_from_dimensionless() {
        let x = (&Number::new(5.0) - &Number::new(10.0)).unwrap();
        assert_eq!(x.value, -5.0);
        assert!(x.is_dimensionless());
    }

    #[test]
    fn number_with_unit_subtracted_from_dimensionless() {
        let result = &Number::new(5.0) - &Number::new(10.0).with_unit((&METER / &SECOND).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn dimensionless_subtracted_from_number_with_unit() {
        let result = &Number::new(10.0).with_unit((&METER / &SECOND).unwrap()) - &Number::new(5.0);
        assert!(result.is_err());
    }

    #[test]
    fn number_with_unit_subtracted_from_compatible_number_with_unit() {
        let x = (&Number::new(10.0).with_unit((&METER / &SECOND).unwrap())
            - &Number::new(5.0).with_unit((&MILE / &HOUR).unwrap()))
            .unwrap();
        assert_eq!(x.value, 7.7648);
        assert_eq!(*x.unit.as_ref().unwrap().numer(), vec![&METER]);
        assert_eq!(*x.unit.unwrap().denom(), vec![&SECOND]);
    }

    #[test]
    fn number_with_unit_subtracted_from_incompatible_number_with_unit() {
        let result = &Number::new(10.0).with_unit((&METER / &SECOND).unwrap())
            - &Number::new(5.0).with_unit((&MILE / &KILOGRAM).unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn dimensionless_multiplied_by_dimensionless() {
        let x = (&Number::new(5.0) * &Number::new(10.0)).unwrap();
        assert_eq!(x.value, 50.0);
        assert!(x.is_dimensionless());
    }

    #[test]
    fn dimensionless_multiplied_by_number_with_unit() {
        let x = (&Number::new(5.0) * &Number::new(10.0).with_unit((&METER / &SECOND).unwrap()))
            .unwrap();
        assert_eq!(x.value, 50.0);
        assert_eq!(*x.unit.as_ref().unwrap().numer(), vec![&METER]);
        assert_eq!(*x.unit.unwrap().denom(), vec![&SECOND]);
    }

    #[test]
    fn number_with_unit_multiplied_by_dimensionless() {
        let x = (&Number::new(5.0).with_unit((&METER / &SECOND).unwrap()) * &Number::new(10.0))
            .unwrap();
        assert_eq!(x.value, 50.0);
        assert_eq!(*x.unit.as_ref().unwrap().numer(), vec![&METER]);
        assert_eq!(*x.unit.unwrap().denom(), vec![&SECOND]);
    }

    #[test]
    fn number_with_unit_multiplied_by_number_with_unit() {
        let x = (&Number::new(5.0).with_unit((&METER / &SECOND).unwrap())
            * &Number::new(10.0).with_unit((&MILE / &HOUR).unwrap()))
            .unwrap();
        assert_eq!(x.value, 50.0);
        assert_eq!(*x.unit.as_ref().unwrap().numer(), vec![&METER, &MILE]);
        assert_eq!(*x.unit.unwrap().denom(), vec![&SECOND, &HOUR]);
    }

    #[test]
    fn dimensionless_multiplied_by_unit() {
        let x = (&Number::new(5.0) * &(&MILE / &HOUR).unwrap()).unwrap();
        assert_eq!(x.value, 5.0);
        assert_eq!(*x.unit.as_ref().unwrap().numer(), vec![&MILE]);
        assert_eq!(*x.unit.unwrap().denom(), vec![&HOUR]);
    }

    #[test]
    fn number_with_unit_multiplied_by_unit() {
        let x = (&Number::new(5.0).with_unit(((&MILE * &HOUR).unwrap() / &KILOGRAM).unwrap())
            * &(&MILE / &HOUR).unwrap())
            .unwrap();
        assert_eq!(x.value, 5.0);
        assert_eq!(*x.unit.as_ref().unwrap().numer(), vec![&MILE, &MILE]);
        assert_eq!(*x.unit.unwrap().denom(), vec![&KILOGRAM]);
    }

    #[test]
    fn number_with_unit_multiplied_by_temperature() {
        let x = &Number::new(5.0).with_unit(((&MILE * &HOUR).unwrap() / &KILOGRAM).unwrap())
            * &TEMP_CELSIUS.as_unit();
        assert!(x.is_err());
    }

    #[test]
    fn dimensionless_divided_by_dimensionless() {
        let x = (&Number::new(5.0) / &Number::new(10.0)).unwrap();
        assert_eq!(x.value, 0.5);
        assert!(x.is_dimensionless());
    }

    #[test]
    fn dimensionless_divided_by_number_with_unit() {
        let x = (&Number::new(5.0) / &Number::new(10.0).with_unit((&METER / &SECOND).unwrap()))
            .unwrap();
        assert_eq!(x.value, 0.5);
        assert_eq!(*x.unit.as_ref().unwrap().numer(), vec![&SECOND]);
        assert_eq!(*x.unit.unwrap().denom(), vec![&METER]);
    }

    #[test]
    fn number_with_unit_divided_by_dimensionless() {
        let x = (&Number::new(5.0).with_unit((&METER / &SECOND).unwrap()) / &Number::new(10.0))
            .unwrap();
        assert_eq!(x.value, 0.5);
        assert_eq!(*x.unit.as_ref().unwrap().numer(), vec![&METER]);
        assert_eq!(*x.unit.unwrap().denom(), vec![&SECOND]);
    }

    #[test]
    fn number_with_unit_divided_by_number_with_unit() {
        let x = (&Number::new(5.0).with_unit((&METER / &SECOND).unwrap())
            / &Number::new(10.0).with_unit((&MILE / &HOUR).unwrap()))
            .unwrap();
        assert_eq!(x.value, 0.5);
        assert_eq!(*x.unit.as_ref().unwrap().numer(), vec![&METER, &HOUR]);
        assert_eq!(*x.unit.unwrap().denom(), vec![&SECOND, &MILE]);
    }

    #[test]
    fn dimensionless_divided_by_unit() {
        let x = (&Number::new(5.0) / &(&MILE / &HOUR).unwrap()).unwrap();
        assert_eq!(x.value, 5.0);
        assert_eq!(*x.unit.as_ref().unwrap().numer(), vec![&HOUR]);
        assert_eq!(*x.unit.unwrap().denom(), vec![&MILE]);
    }

    #[test]
    fn number_with_unit_divided_by_unit() {
        let x = (&Number::new(5.0).with_unit(((&MILE * &HOUR).unwrap() / &KILOGRAM).unwrap())
            / &(&MILE / &HOUR).unwrap())
            .unwrap();
        assert_eq!(x.value, 5.0);
        assert_eq!(*x.unit.as_ref().unwrap().numer(), vec![&HOUR, &HOUR]);
        assert_eq!(*x.unit.unwrap().denom(), vec![&KILOGRAM]);
    }

    #[test]
    fn number_with_unit_divided_by_temperature() {
        let x = &Number::new(5.0).with_unit(((&MILE * &HOUR).unwrap() / &KILOGRAM).unwrap())
            / &TEMP_CELSIUS.as_unit();
        assert!(x.is_err());
    }

    #[test]
    fn display_dimensionless_with_exponent_format() {
        // six decimal places max
        assert_eq!(Number::new(0.000898359204909915).to_string(), "8.983592e-4");
        assert_eq!(
            Number::new(4180506471207144.8470604546950069).to_string(),
            "4.180506e15"
        );
        // trim trailing zeroes
        assert_eq!(Number::new(0.0000442).to_string(), "4.42e-5");
        assert_eq!(
            Number::new(5821600000000000.3253253941312786).to_string(),
            "5.8216e15"
        );
        // trim trailing zeroes and decimal point
        assert_eq!(Number::new(0.0004).to_string(), "4e-4");
        assert_eq!(
            Number::new(2000000000000.8142598874151412).to_string(),
            "2e12"
        );
        // again, but negative
        assert_eq!(
            Number::new(-0.000898359204909915).to_string(),
            "-8.983592e-4"
        );
        assert_eq!(
            Number::new(-4180506471207144.8470604546950069).to_string(),
            "-4.180506e15"
        );
        assert_eq!(Number::new(-0.0000442).to_string(), "-4.42e-5");
        assert_eq!(
            Number::new(-5821600000000000.3253253941312786).to_string(),
            "-5.8216e15"
        );
        assert_eq!(Number::new(-0.0004).to_string(), "-4e-4");
        assert_eq!(
            Number::new(-2000000000000.8142598874151412).to_string(),
            "-2e12"
        );
    }

    #[test]
    fn display_dimensionless_with_decimal_format() {
        // make sure the basics work
        assert_eq!(Number::new(0.0).to_string(), "0");
        assert_eq!(Number::new(1.0).to_string(), "1");
        // six decimal places max
        assert_eq!(Number::new(0.0027442391822086665).to_string(), "0.002744");
        assert_eq!(Number::new(932.9624592477858).to_string(), "932.962459");
        // trim trailing zeroes
        assert_eq!(Number::new(0.0084).to_string(), "0.0084");
        assert_eq!(Number::new(804.2737).to_string(), "804.2737");
        // trim trailing zeroes and decimal point
        assert_eq!(Number::new(600.0).to_string(), "600");
        // again, but negative
        assert_eq!(Number::new(-0.0).to_string(), "0");
        assert_eq!(Number::new(-1.0).to_string(), "-1");
        // six decimal places max
        assert_eq!(Number::new(-0.0027442391822086665).to_string(), "-0.002744");
        assert_eq!(Number::new(-932.9624592477858).to_string(), "-932.962459");
        // trim trailing zeroes
        assert_eq!(Number::new(-0.0084).to_string(), "-0.0084");
        assert_eq!(Number::new(-804.2737).to_string(), "-804.2737");
        // trim trailing zeroes and decimal point
        assert_eq!(Number::new(-600.0).to_string(), "-600");
    }

    #[test]
    fn display_with_units_with_exponent_format() {
        let u = (&METER / &SECOND).unwrap();
        // six decimal places max
        assert_eq!(
            Number::new(0.000898359204909915)
                .with_unit(u.clone())
                .to_string(),
            "[8.983592e-4 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(4180506471207144.8470604546950069)
                .with_unit(u.clone())
                .to_string(),
            "[4.180506e15 m⋅s⁻¹]"
        );
        // trim trailing zeroes
        assert_eq!(
            Number::new(0.0000442).with_unit(u.clone()).to_string(),
            "[4.42e-5 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(5821600000000000.3253253941312786)
                .with_unit(u.clone())
                .to_string(),
            "[5.8216e15 m⋅s⁻¹]"
        );
        // trim trailing zeroes and decimal point
        assert_eq!(
            Number::new(0.0004).with_unit(u.clone()).to_string(),
            "[4e-4 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(2000000000000.8142598874151412)
                .with_unit(u.clone())
                .to_string(),
            "[2e12 m⋅s⁻¹]"
        );
        // again, but negative
        assert_eq!(
            Number::new(-0.000898359204909915)
                .with_unit(u.clone())
                .to_string(),
            "[-8.983592e-4 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(-4180506471207144.8470604546950069)
                .with_unit(u.clone())
                .to_string(),
            "[-4.180506e15 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(-0.0000442).with_unit(u.clone()).to_string(),
            "[-4.42e-5 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(-5821600000000000.3253253941312786)
                .with_unit(u.clone())
                .to_string(),
            "[-5.8216e15 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(-0.0004).with_unit(u.clone()).to_string(),
            "[-4e-4 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(-2000000000000.8142598874151412)
                .with_unit(u.clone())
                .to_string(),
            "[-2e12 m⋅s⁻¹]"
        );
    }

    #[test]
    fn display_with_units_with_decimal_format() {
        let u = (&METER / &SECOND).unwrap();
        // make sure the basics work
        assert_eq!(
            Number::new(0.0).with_unit(u.clone()).to_string(),
            "[0 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(1.0).with_unit(u.clone()).to_string(),
            "[1 m⋅s⁻¹]"
        );
        // six decimal places max
        assert_eq!(
            Number::new(0.0027442391822086665)
                .with_unit(u.clone())
                .to_string(),
            "[0.002744 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(932.9624592477858)
                .with_unit(u.clone())
                .to_string(),
            "[932.962459 m⋅s⁻¹]"
        );
        // trim trailing zeroes
        assert_eq!(
            Number::new(0.0084).with_unit(u.clone()).to_string(),
            "[0.0084 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(804.2737).with_unit(u.clone()).to_string(),
            "[804.2737 m⋅s⁻¹]"
        );
        // trim trailing zeroes and decimal point
        assert_eq!(
            Number::new(600.0).with_unit(u.clone()).to_string(),
            "[600 m⋅s⁻¹]"
        );
        // again, but negative
        assert_eq!(
            Number::new(-0.0).with_unit(u.clone()).to_string(),
            "[0 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(-1.0).with_unit(u.clone()).to_string(),
            "[-1 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(-0.0027442391822086665)
                .with_unit(u.clone())
                .to_string(),
            "[-0.002744 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(-932.9624592477858)
                .with_unit(u.clone())
                .to_string(),
            "[-932.962459 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(-0.0084).with_unit(u.clone()).to_string(),
            "[-0.0084 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(-804.2737).with_unit(u.clone()).to_string(),
            "[-804.2737 m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(-600.0).with_unit(u.clone()).to_string(),
            "[-600 m⋅s⁻¹]"
        );
    }

    #[test]
    fn display_special_values() {
        assert_eq!(Number::new(f64::NAN).to_string(), "NaN");
        assert_eq!(Number::new(f64::INFINITY).to_string(), "inf");
        assert_eq!(Number::new(f64::NEG_INFINITY).to_string(), "-inf");
        // again, but with units
        let u = (&METER / &SECOND).unwrap();
        assert_eq!(
            Number::new(f64::NAN).with_unit(u.clone()).to_string(),
            "[NaN m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(f64::INFINITY).with_unit(u.clone()).to_string(),
            "[inf m⋅s⁻¹]"
        );
        assert_eq!(
            Number::new(f64::NEG_INFINITY)
                .with_unit(u.clone())
                .to_string(),
            "[-inf m⋅s⁻¹]"
        );
    }
}
