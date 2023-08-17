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

use super::{base::NUM_PHYSICAL_QUANTITIES, Base, Error};
#[allow(clippy::enum_glob_use)]
use Error::*;

/// A unit that may measure a base quantity or a derived quantity.
#[derive(Clone, Debug)]
pub struct Unit {
    /// Symbolic representation of the unit
    pub symbol: Option<String>,
    #[doc(hidden)]
    pub numer: Vec<&'static Base>,
    #[doc(hidden)]
    pub denom: Vec<&'static Base>,
}

impl Unit {
    /// Derives a unit from the given base units.
    ///
    /// # Parameters
    ///
    /// - `numer`: Base units with positive exponents
    /// - `denom`: Base units with negative exponents
    ///
    /// The number of times a base unit appears in `numer` or `denom` determines
    /// the magnitude of its exponent.
    ///
    /// If any of the base units cancel out, they are removed from the result.
    #[must_use]
    pub fn new(numer: &[&'static Base], denom: &[&'static Base]) -> Self {
        Unit {
            symbol: None,
            numer: Vec::from(numer),
            denom: Vec::from(denom),
        }
        .simplified()
    }

    /// Returns a new `Unit` identical to this one except that it has the given
    /// symbol.
    #[must_use]
    pub fn with_symbol(&self, symbol: &str) -> Self {
        Unit {
            symbol: Some(String::from(symbol)),
            numer: self.numer.clone(),
            denom: self.denom.clone(),
        }
    }

    /// Converts a number in this unit to another unit, without adjusting for
    /// differences in zero points. This function is only suitable for
    /// converting intervals/deltas, not absolutes.
    ///
    /// # Errors
    ///
    /// Returns an error if `self` can't be converted to `other`.
    pub fn convert_interval(&self, num: f64, other: &Self) -> Result<f64, Error> {
        let mut num = num;

        if !self.is_commensurable_with(other) {
            return Err(IncommensurableUnits(
                Some(Box::new(self.clone())),
                Some(Box::new(other.clone())),
            ));
        }

        // Reduce to SI
        for base in &self.numer {
            num *= base.factor;
        }
        for base in &self.denom {
            num /= base.factor;
        }

        // Raise to new unit
        for base in &other.numer {
            num /= base.factor;
        }
        for base in &other.denom {
            num *= base.factor;
        }

        Ok(num)
    }

    /// Converts a number in this unit to another unit, adjusting for
    /// differences in zero points.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `self` can't be converted to `other`; or
    /// - the unit does not consist of a single base unit with an exponent of
    ///   positive one.
    pub fn convert_absolute(&self, num: f64, other: &Self) -> Result<f64, Error> {
        let mut num = num;

        if self.numer.len() + self.denom.len() > 1 {
            return Err(TooManyBaseUnits(Box::new(self.clone())));
        }
        if other.numer.len() + self.denom.len() > 1 {
            return Err(TooManyBaseUnits(Box::new(other.clone())));
        }
        if !self.denom.is_empty() {
            return Err(NegativeExponent(Box::new(self.clone())));
        }
        if !other.denom.is_empty() {
            return Err(NegativeExponent(Box::new(other.clone())));
        }

        if !self.is_commensurable_with(other) {
            return Err(IncommensurableUnits(
                Some(Box::new(self.clone())),
                Some(Box::new(other.clone())),
            ));
        }

        num -= self.numer[0].zero;
        num *= self.numer[0].factor;
        num = num.mul_add(other.numer[0].factor.recip(), other.numer[0].zero);

        Ok(num)
    }

    /// Returns a unit with the same base units as this one, but with all of the
    /// exponents multiplied by -1.
    #[must_use]
    pub fn inverse(&self) -> Unit {
        Self::new(self.denom.as_slice(), self.numer.as_slice())
    }

    /// Helper function for `is_commensurable_with`. Returns true if each
    /// physical quantity occurs the same number of times in both sequences.
    fn physq_counts_match(a: &Vec<&'static Base>, b: &Vec<&'static Base>) -> bool {
        let mut counts = (
            [0_usize; NUM_PHYSICAL_QUANTITIES],
            [0_usize; NUM_PHYSICAL_QUANTITIES],
        );

        for base in a {
            counts.0[base.physq as usize] += 1;
        }
        for base in b {
            counts.1[base.physq as usize] += 1;
        }

        for i in 0..NUM_PHYSICAL_QUANTITIES {
            if counts.0[i] != counts.1[i] {
                return false;
            }
        }

        true
    }

    /// Determines whether a quantity in this unit can be converted to another unit.
    #[must_use]
    pub fn is_commensurable_with(&self, other: &Unit) -> bool {
        // If each physical sequence occurs the same number of times in this
        // unit's numerator and the other unit's numerator, and the same is true
        // of the denominators, then the units are commensurable.
        Unit::physq_counts_match(&self.numer, &other.numer)
            && Unit::physq_counts_match(&self.denom, &other.denom)
    }

    /// Returns a new `Unit` mathematically identical to this one but without
    /// any base units that cancel each other out.
    fn simplified(&self) -> Self {
        let mut s_numer = Vec::from(self.numer.as_slice());
        let mut s_denom = Vec::from(self.denom.as_slice());
        let mut numer_ix = 0;
        let mut should_incr: bool;

        while numer_ix < s_numer.len() {
            should_incr = true;

            for denom_ix in 0..s_denom.len() {
                if s_numer[numer_ix] == s_denom[denom_ix] {
                    s_numer.remove(numer_ix);
                    s_denom.remove(denom_ix);
                    should_incr = false;
                    break;
                }
            }

            if should_incr {
                numer_ix += 1;
            }
        }

        Unit {
            symbol: self.symbol.clone(),
            numer: s_numer,
            denom: s_denom,
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        if let Some(symbol) = &self.symbol {
            return fmt.write_fmt(format_args!("{symbol}"));
        }

        let mut symbols = Vec::new();
        symbols.extend(self.numer.iter().map(|b| b.symbol));

        fmt.write_fmt(format_args!("{}", symbols.join("⋅")))
            .and_then(|_| {
                if self.denom.is_empty() {
                    Ok(())
                } else {
                    let prefix = if self.numer.is_empty() { "" } else { "⋅" };
                    symbols.clear();
                    symbols.extend(self.denom.iter().map(|b| b.symbol));
                    fmt.write_fmt(format_args!("{prefix}{}⁻¹", symbols.join("⁻¹⋅")))
                }
            })
    }
}

impl std::ops::Mul<Self> for &Unit {
    type Output = Unit;

    /// Produces the unit that would result from multiplying a quantity in this
    /// unit with a quantity in another unit.
    fn mul(self, other: &Unit) -> Unit {
        let mut numer = self.numer.clone();
        let mut denom = self.denom.clone();
        numer.extend(&other.numer);
        denom.extend(&other.denom);
        Unit::new(numer.as_slice(), denom.as_slice())
    }
}

impl std::ops::Mul<&'static Base> for Unit {
    type Output = Unit;

    /// Produces the unit that would result from multiplying a quantity in this
    /// unit with a quantity in a base unit.
    fn mul(self, other: &'static Base) -> Unit {
        &self * other
    }
}

impl std::ops::Mul<&'static Base> for &Unit {
    type Output = Unit;

    /// Produces the unit that would result from multiplying a quantity in this
    /// unit with a quantity in a base unit.
    fn mul(self, other: &'static Base) -> Unit {
        let mut numer = self.numer.clone();
        numer.extend([other]);
        Unit::new(numer.as_slice(), self.denom.as_slice())
    }
}

impl std::ops::Div<Self> for &Unit {
    type Output = Unit;

    /// Produces the unit that would result from dividing a quantity in this
    /// unit by a quantity in another unit.
    fn div(self, other: &Unit) -> Unit {
        let mut numer = self.numer.clone();
        let mut denom = self.denom.clone();
        numer.extend(&other.denom);
        denom.extend(&other.numer);
        Unit::new(numer.as_slice(), denom.as_slice())
    }
}

impl std::ops::Div<&'static Base> for Unit {
    type Output = Unit;

    /// Produces the unit that would result from dividing a quantity in this
    /// unit by a quantity in a base unit.
    fn div(self, other: &'static Base) -> Unit {
        &self / other
    }
}

impl std::ops::Div<&'static Base> for &Unit {
    type Output = Unit;

    /// Produces the unit that would result from dividing a quantity in this
    /// unit by a quantity in a base unit.
    fn div(self, other: &'static Base) -> Unit {
        let mut denom = self.denom.clone();
        denom.extend([other]);
        Unit::new(self.numer.as_slice(), denom.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use crate::units::{
        Unit, AMPERE, CELSIUS, FAHRENHEIT, FOOT, HOUR, KILOGRAM, METER, MILE, NAUTICAL_MILE, SECOND,
    };
    use approx::assert_relative_eq;

    #[test]
    fn unit_display() {
        let m_kg_per_ampere_s = &METER * &KILOGRAM / &AMPERE / &SECOND;
        assert_eq!(m_kg_per_ampere_s.to_string(), "m⋅kg⋅A⁻¹⋅s⁻¹");

        let joule = Unit {
            symbol: Some(String::from("J")),
            numer: vec![&KILOGRAM, &METER, &METER],
            denom: vec![&SECOND, &SECOND],
        };
        assert_eq!(joule.to_string(), "J");
    }

    #[test]
    fn unit_multiplied_by_unit() {
        let m_per_s = &METER / &SECOND;
        let kg_per_amp = &KILOGRAM / &AMPERE;
        let result = &m_per_s * &kg_per_amp;
        assert_eq!(result.numer, vec![&METER, &KILOGRAM]);
        assert_eq!(result.denom, vec![&SECOND, &AMPERE]);
    }

    #[test]
    fn unit_multiplied_by_base() {
        let m_per_s = &METER / &SECOND;
        let result = m_per_s * &KILOGRAM;
        assert_eq!(result.numer, vec![&METER, &KILOGRAM]);
        assert_eq!(result.denom, vec![&SECOND]);
    }

    #[test]
    fn unit_divided_by_unit() {
        let m_per_s = &METER / &SECOND;
        let kg_per_amp = &KILOGRAM / &AMPERE;
        let result = &m_per_s / &kg_per_amp;
        assert_eq!(result.numer, vec![&METER, &AMPERE]);
        assert_eq!(result.denom, vec![&SECOND, &KILOGRAM]);
    }

    #[test]
    fn unit_divided_by_base() {
        let m_per_s = &METER / &SECOND;
        let result = m_per_s / &KILOGRAM;
        assert_eq!(result.numer, vec![&METER]);
        assert_eq!(result.denom, vec![&SECOND, &KILOGRAM]);
    }

    #[test]
    fn unit_simplification() {
        let kg_per_s = &SECOND * &METER * &KILOGRAM / &METER / &SECOND / &SECOND;
        assert_eq!(kg_per_s.numer, vec![&KILOGRAM]);
        assert_eq!(kg_per_s.denom, vec![&SECOND]);
    }

    #[test]
    fn interval_unit_conversion() {
        let m = Unit::new(&[&METER], &[]);
        let ft = Unit::new(&[&FOOT], &[]);
        assert_eq!(m.convert_interval(7.0, &ft).unwrap(), 7.0 / 0.3048);

        let mph = Unit::new(&[&MILE], &[&HOUR]);
        let kts = Unit::new(&[&NAUTICAL_MILE], &[&HOUR]);
        assert_eq!(
            mph.convert_interval(110.0, &kts).unwrap(),
            110.0 * 1609.344 / 1852.0
        );

        let m_per_s = Unit::new(&[&METER], &[&SECOND]);
        let hz = Unit::new(&[], &[&SECOND]);
        assert!(m_per_s.convert_interval(1.0, &hz).is_err());

        let c = Unit::new(&[&CELSIUS], &[]);
        let f = Unit::new(&[&FAHRENHEIT], &[]);
        assert_relative_eq!(c.convert_interval(1.0, &f).unwrap(), 9.0 / 5.0);

        let c = Unit::new(&[], &[&CELSIUS]);
        let f = Unit::new(&[], &[&FAHRENHEIT]);
        assert_relative_eq!(c.convert_interval(1.0, &f).unwrap(), 5.0 / 9.0);
    }

    #[test]
    fn absolute_unit_conversion() {
        let m = Unit::new(&[&METER], &[]);
        let ft = Unit::new(&[&FOOT], &[]);
        assert_eq!(m.convert_absolute(7.0, &ft).unwrap(), 7.0 / 0.3048);

        let mph = Unit::new(&[&MILE], &[&HOUR]);
        let kts = Unit::new(&[&NAUTICAL_MILE], &[&HOUR]);
        assert!(mph.convert_absolute(110.0, &kts).is_err());

        let m_per_s = Unit::new(&[&METER], &[&SECOND]);
        let hz = Unit::new(&[], &[&SECOND]);
        assert!(m_per_s.convert_absolute(1.0, &hz).is_err());

        let c = Unit::new(&[&CELSIUS], &[]);
        let f = Unit::new(&[&FAHRENHEIT], &[]);
        assert_relative_eq!(
            c.convert_absolute(1.0, &f).unwrap(),
            33.8,
            epsilon = f64::EPSILON * 448.0
        ); // 9.947598300641403e-14

        let c = Unit::new(&[], &[&CELSIUS]);
        let f = Unit::new(&[], &[&FAHRENHEIT]);
        assert!(c.convert_absolute(1.0, &f).is_err());
    }
}
