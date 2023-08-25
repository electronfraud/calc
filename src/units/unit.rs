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

use super::{base::NUM_PHYSICAL_QUANTITIES, Base, Error};
#[allow(clippy::enum_glob_use)]
use Error::*;

/// A unit that may measure a base quantity or a derived quantity.
#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    /// Symbolic representation of the unit
    pub symbol: Option<String>,
    numer: Vec<Base>,
    denom: Vec<Base>,
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
    ///
    /// # Errors
    ///
    /// Returns an error if one of the base units has a non-zero zero point and:
    /// - there is more than one base unit; or,
    /// - the denominator is not empty.
    pub fn new(numer: &[Base], denom: &[Base]) -> Result<Self, Error> {
        let u = Unit {
            symbol: None,
            numer: Vec::from(numer),
            denom: Vec::from(denom),
        }
        .simplified();

        for base in &u.numer {
            if base.zero.is_some()
                && base.zero != Some(0.0)
                && (u.numer.len() > 1 || !u.denom.is_empty())
            {
                return Err(NonzeroZeroPoint(*base));
            }
        }

        for base in &u.denom {
            if base.zero.is_some() && base.zero != Some(0.0) {
                return Err(NonzeroZeroPoint(*base));
            }
        }

        Ok(u)
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

    /// Return a vector of this unit's base units that have positive exponents.
    /// The number of times a base unit appears in the vector indicates the
    /// magnitude of its exponent.
    #[must_use]
    pub fn numer(&self) -> &Vec<Base> {
        &self.numer
    }

    /// Return a vector of this unit's base units that have negative exponents.
    /// The number of times a base unit appears in the vector indicates the
    /// magnitude of its exponent.
    #[must_use]
    pub fn denom(&self) -> &Vec<Base> {
        &self.denom
    }

    /// Converts a number in this unit to another unit.
    ///
    /// # Errors
    ///
    /// Returns an error if `self` can't be converted to `other`.
    pub fn convert(&self, num: f64, other: &Self) -> Result<f64, Error> {
        let mut num = num;

        if !self.is_commensurable_with(other) {
            return Err(IncommensurableUnits(
                Some(Box::new(self.clone())),
                Some(Box::new(other.clone())),
            ));
        }

        // Reduce to SI
        for base in &self.numer {
            if let Some(z) = base.zero {
                num -= z;
            }
            num *= base.factor;
        }
        for base in &self.denom {
            num /= base.factor;
        }

        // Raise to new unit
        for base in &other.numer {
            num /= base.factor;
            if let Some(z) = base.zero {
                num += z;
            }
        }
        for base in &other.denom {
            num *= base.factor;
        }

        Ok(num)
    }

    /// Returns a unit with the same base units as this one, but with all of the
    /// exponents multiplied by -1.
    ///
    /// # Errors
    ///
    /// Returns an error if the unit has a zero point. Inversion of these units
    /// is nonsensical.
    pub fn inverse(&self) -> Result<Self, Error> {
        if !self.numer.is_empty() && self.numer[0].zero.is_some() && self.numer[0].zero != Some(0.0)
        {
            return Err(UninvertableUnits(Box::new(self.clone())));
        }
        Self::new(self.denom.as_slice(), self.numer.as_slice())
    }

    /// Helper function for `is_commensurable_with`. Returns true if each
    /// physical quantity occurs the same number of times in both sequences.
    fn physq_counts_match(a: &Vec<Base>, b: &Vec<Base>) -> bool {
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
        // If number of occurrences of each physical quantity in the numerators
        // differs, then the units are incommensurable; likewise for the
        // denominators.
        if !(Unit::physq_counts_match(&self.numer, &other.numer)
            && Unit::physq_counts_match(&self.denom, &other.denom))
        {
            return false;
        }

        // By contract we assume that any unit with a temperature base (z != 0)
        // consists solely of that base, in the numerator. If that isn't the
        // case with these units, there's nothing left to check.
        if !(self.numer.len() == 1 && self.denom.is_empty()) {
            return true;
        }

        let a = self.numer[0].zero;
        let b = other.numer[0].zero;

        // Conversion to or from Kelvin or Rankine is always allowed.
        if a == Some(0.0) || b == Some(0.0) {
            return true;
        }

        // Otherwise both units must have a zero, or both units must not have a
        // zero.
        (a.is_some() && b.is_some()) || (a.is_none() && b.is_none())
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

const SUPERSCRIPTS: [&str; 10] = ["⁰", "¹", "²", "³", "⁴", "⁵", "⁶", "⁷", "⁸", "⁹"];

/// Turns an integer `i` into a string using superscript digits.
fn i_to_str_superscripts(i: usize) -> String {
    let mut result = String::new();
    for ch in i.to_string().chars() {
        result.push_str(SUPERSCRIPTS[(ch as usize) - ('0' as usize)]);
    }
    result
}

/// Given a sequence of bases, generates a string like "m²⋅A¹⋅s¹". Each exponent
/// is prefixed with `sign`.
fn bases_to_string(bases: &[Base], sign: Option<char>) -> Option<String> {
    if bases.is_empty() {
        return None;
    }

    // Count the number of times each base occurs, preserving the order in
    // in which each base is first encountered.
    let mut uniq_bases: Vec<Base> = Vec::new();
    let mut counts: Vec<usize> = Vec::new();

    for base in bases {
        let ix = uniq_bases
            .iter()
            .position(|b| b == base)
            .unwrap_or_else(|| {
                uniq_bases.push(*base);
                counts.push(0);
                uniq_bases.len() - 1
            });
        counts[ix] += 1;
    }

    // Generate a string containing each base and its exponent.
    let mut result = String::new();

    for ix in 0..uniq_bases.len() {
        result.push_str(uniq_bases[ix].symbol);
        if let Some(sign) = sign {
            result.push(sign);
        }
        if counts[ix] > 1 || sign.is_some() {
            result.push_str(&i_to_str_superscripts(counts[ix]));
        }
        result.push('⋅');
    }

    result.pop();
    Some(result)
}

impl std::fmt::Display for Unit {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        // If the unit has an assigned symbol, just use that.
        if let Some(symbol) = &self.symbol {
            return fmt.write_fmt(format_args!("{symbol}"));
        }

        // Otherwise, generate a string with the symbol's bases and exponents.
        let pos = bases_to_string(&self.numer, None);
        let neg = bases_to_string(&self.denom, Some('⁻'));

        match (pos, neg) {
            (Some(pos), Some(neg)) => write!(fmt, "{pos}⋅{neg}"),
            (Some(pos), None) => write!(fmt, "{pos}"),
            (None, Some(neg)) => write!(fmt, "{neg}"),
            (None, None) => panic!("Unit with empty `numer` and `denom`"),
        }
    }
}

impl std::ops::Mul<Self> for &Unit {
    type Output = Result<Unit, Error>;

    /// Produces the unit that would result from multiplying a quantity in this
    /// unit with a quantity in another unit.
    fn mul(self, other: &Unit) -> Result<Unit, Error> {
        let mut numer = self.numer.clone();
        let mut denom = self.denom.clone();
        numer.extend(&other.numer);
        denom.extend(&other.denom);
        Unit::new(numer.as_slice(), denom.as_slice())
    }
}

impl std::ops::Mul<Base> for Unit {
    type Output = Result<Unit, Error>;

    /// Produces the unit that would result from multiplying a quantity in this
    /// unit with a quantity in a base unit.
    fn mul(self, other: Base) -> Result<Unit, Error> {
        &self * other
    }
}

impl std::ops::Mul<Base> for &Unit {
    type Output = Result<Unit, Error>;

    /// Produces the unit that would result from multiplying a quantity in this
    /// unit with a quantity in a base unit.
    fn mul(self, other: Base) -> Result<Unit, Error> {
        let mut numer = self.numer.clone();
        numer.extend([other]);
        Unit::new(numer.as_slice(), self.denom.as_slice())
    }
}

impl std::ops::Div<Self> for &Unit {
    type Output = Result<Unit, Error>;

    /// Produces the unit that would result from dividing a quantity in this
    /// unit by a quantity in another unit.
    fn div(self, other: &Unit) -> Result<Unit, Error> {
        let mut numer = self.numer.clone();
        let mut denom = self.denom.clone();
        numer.extend(&other.denom);
        denom.extend(&other.numer);
        Unit::new(numer.as_slice(), denom.as_slice())
    }
}

impl std::ops::Div<Base> for Unit {
    type Output = Result<Unit, Error>;

    /// Produces the unit that would result from dividing a quantity in this
    /// unit by a quantity in a base unit.
    fn div(self, other: Base) -> Result<Unit, Error> {
        &self / other
    }
}

impl std::ops::Div<Base> for &Unit {
    type Output = Result<Unit, Error>;

    /// Produces the unit that would result from dividing a quantity in this
    /// unit by a quantity in a base unit.
    fn div(self, other: Base) -> Result<Unit, Error> {
        let mut denom = self.denom.clone();
        denom.extend([other]);
        Unit::new(self.numer.as_slice(), denom.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::units::Unit;
    use crate::units::{
        AMPERE, DEG_CELSIUS, DEG_FAHRENHEIT, FOOT, HOUR, KELVIN, KILOGRAM, METER, MILE,
        NAUTICAL_MILE, RANKINE, SECOND, TEMP_CELSIUS, TEMP_FAHRENHEIT,
    };

    #[test]
    fn unit_display() {
        let m_kg_per_ampere_s = (((METER * KILOGRAM).unwrap() / AMPERE).unwrap() / SECOND).unwrap();
        assert_eq!(m_kg_per_ampere_s.to_string(), "m⋅kg⋅A⁻¹⋅s⁻¹");

        let joule = Unit {
            symbol: Some(String::from("J")),
            numer: vec![KILOGRAM, METER, METER],
            denom: vec![SECOND, SECOND],
        };
        assert_eq!(joule.to_string(), "J");

        let joule = Unit {
            symbol: None,
            numer: vec![KILOGRAM, METER, METER],
            denom: vec![SECOND, SECOND],
        };
        assert_ne!(joule.to_string(), "J");
        assert_eq!(joule.with_symbol("J").to_string(), "J");
    }

    #[test]
    fn unit_display_exponents() {
        let u = (((METER * METER).unwrap() / AMPERE).unwrap() / SECOND).unwrap();
        assert_eq!(u.to_string(), "m²⋅A⁻¹⋅s⁻¹");
        let u = (&u / SECOND).unwrap();
        assert_eq!(u.to_string(), "m²⋅A⁻¹⋅s⁻²");
        let u = Unit::new(&[SECOND, SECOND, AMPERE], &[]).unwrap();
        assert_eq!(u.to_string(), "s²⋅A");
        let u = Unit::new(&[], &[SECOND, SECOND, AMPERE]).unwrap();
        assert_eq!(u.to_string(), "s⁻²⋅A⁻¹");
    }

    #[test]
    fn unit_multiplied_by_unit() {
        let m_per_s = (METER / SECOND).unwrap();
        let kg_per_amp = (KILOGRAM / AMPERE).unwrap();
        let result = (&m_per_s * &kg_per_amp).unwrap();
        assert_eq!(result.numer, vec![METER, KILOGRAM]);
        assert_eq!(result.denom, vec![SECOND, AMPERE]);
    }

    #[test]
    fn unit_multiplied_by_base() {
        let m_per_s = (METER / SECOND).unwrap();
        let result = (m_per_s * KILOGRAM).unwrap();
        assert_eq!(result.numer, vec![METER, KILOGRAM]);
        assert_eq!(result.denom, vec![SECOND]);
    }

    #[test]
    fn unit_divided_by_unit() {
        let m_per_s = (METER / SECOND).unwrap();
        let kg_per_amp = (KILOGRAM / AMPERE).unwrap();
        let result = (&m_per_s / &kg_per_amp).unwrap();
        assert_eq!(result.numer, vec![METER, AMPERE]);
        assert_eq!(result.denom, vec![SECOND, KILOGRAM]);
    }

    #[test]
    fn unit_divided_by_base() {
        let m_per_s = (METER / SECOND).unwrap();
        let result = (m_per_s / KILOGRAM).unwrap();
        assert_eq!(result.numer, vec![METER]);
        assert_eq!(result.denom, vec![SECOND, KILOGRAM]);
    }

    #[test]
    fn unit_simplification() {
        let kg_per_s = (((((SECOND * METER).unwrap() * KILOGRAM).unwrap() / METER).unwrap()
            / SECOND)
            .unwrap()
            / SECOND)
            .unwrap();
        assert_eq!(kg_per_s.numer, vec![KILOGRAM]);
        assert_eq!(kg_per_s.denom, vec![SECOND]);
    }

    #[test]
    fn unit_conversion() {
        let m = Unit::new(&[METER], &[]).unwrap();
        let ft = Unit::new(&[FOOT], &[]).unwrap();
        assert_eq!(m.convert(7.0, &ft).unwrap(), 7.0 / 0.3048);

        let mph = Unit::new(&[MILE], &[HOUR]).unwrap();
        let kts = Unit::new(&[NAUTICAL_MILE], &[HOUR]).unwrap();
        assert_eq!(mph.convert(110.0, &kts).unwrap(), 110.0 * 1609.344 / 1852.0);

        let m_per_s = Unit::new(&[METER], &[SECOND]).unwrap();
        let hz = Unit::new(&[], &[SECOND]).unwrap();
        assert!(m_per_s.convert(1.0, &hz).is_err());
    }

    //     Acceptable temperature conversions
    // ------------------------------------------
    //                     TO
    //          tempC tempF  degC  degF   K     R
    //    tempC   o     o     x     x     o     o
    // F  tempF   o     o     x     x     o     o
    // R   degC   x     x     o     o     o     o
    // O   degF   x     x     o     o     o     o
    // M      K   o     o     o     o     o     o
    //        R   o     o     o     o     o     o

    #[test]
    fn temp_celsius_conversions() {
        let a = Unit::new(&[TEMP_CELSIUS], &[]).unwrap();
        let b = Unit::new(&[TEMP_CELSIUS], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 1.0);
        let a = Unit::new(&[TEMP_CELSIUS], &[]).unwrap();
        let b = Unit::new(&[TEMP_FAHRENHEIT], &[]).unwrap();
        assert_relative_eq!(
            a.convert(1.0, &b).unwrap(),
            33.8,
            epsilon = f64::EPSILON * 448.0
        );
        let a = Unit::new(&[TEMP_CELSIUS], &[]).unwrap();
        let b = Unit::new(&[DEG_CELSIUS], &[]).unwrap();
        assert!(a.convert(1.0, &b).is_err());
        let a = Unit::new(&[TEMP_CELSIUS], &[]).unwrap();
        let b = Unit::new(&[DEG_FAHRENHEIT], &[]).unwrap();
        assert!(a.convert(1.0, &b).is_err());
        let a = Unit::new(&[TEMP_CELSIUS], &[]).unwrap();
        let b = Unit::new(&[KELVIN], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 274.15);
        let a = Unit::new(&[TEMP_CELSIUS], &[]).unwrap();
        let b = Unit::new(&[RANKINE], &[]).unwrap();
        assert_relative_eq!(
            a.convert(1.0, &b).unwrap(),
            493.47,
            epsilon = f64::EPSILON * 512.0
        );
    }

    #[test]
    fn temp_fahrenheit_conversions() {
        let a = Unit::new(&[TEMP_FAHRENHEIT], &[]).unwrap();
        let b = Unit::new(&[TEMP_CELSIUS], &[]).unwrap();
        assert_relative_eq!(
            a.convert(1.0, &b).unwrap(),
            (1.0 - 32.0) * 5.0 / 9.0,
            epsilon = f64::EPSILON * 224.0
        );
        let a = Unit::new(&[TEMP_FAHRENHEIT], &[]).unwrap();
        let b = Unit::new(&[TEMP_FAHRENHEIT], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 1.0);
        let a = Unit::new(&[TEMP_FAHRENHEIT], &[]).unwrap();
        let b = Unit::new(&[DEG_CELSIUS], &[]).unwrap();
        assert!(a.convert(1.0, &b).is_err());
        let a = Unit::new(&[TEMP_FAHRENHEIT], &[]).unwrap();
        let b = Unit::new(&[DEG_FAHRENHEIT], &[]).unwrap();
        assert!(a.convert(1.0, &b).is_err());
        let a = Unit::new(&[TEMP_FAHRENHEIT], &[]).unwrap();
        let b = Unit::new(&[KELVIN], &[]).unwrap();
        assert_relative_eq!(
            a.convert(1.0, &b).unwrap(),
            273.15 + (1.0 - 32.0) * 5.0 / 9.0,
            epsilon = f64::EPSILON * 256.0
        );
        let a = Unit::new(&[TEMP_FAHRENHEIT], &[]).unwrap();
        let b = Unit::new(&[RANKINE], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 460.67);
    }

    #[test]
    fn deg_celsius_conversions() {
        let a = Unit::new(&[DEG_CELSIUS], &[]).unwrap();
        let b = Unit::new(&[TEMP_CELSIUS], &[]).unwrap();
        assert!(a.convert(1.0, &b).is_err());
        let a = Unit::new(&[DEG_CELSIUS], &[]).unwrap();
        let b = Unit::new(&[TEMP_FAHRENHEIT], &[]).unwrap();
        assert!(a.convert(1.0, &b).is_err());
        let a = Unit::new(&[DEG_CELSIUS], &[]).unwrap();
        let b = Unit::new(&[DEG_CELSIUS], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 1.0);
        let a = Unit::new(&[DEG_CELSIUS], &[]).unwrap();
        let b = Unit::new(&[DEG_FAHRENHEIT], &[]).unwrap();
        assert_relative_eq!(a.convert(1.0, &b).unwrap(), 1.8);
        let a = Unit::new(&[DEG_CELSIUS], &[]).unwrap();
        let b = Unit::new(&[KELVIN], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 1.0);
        let a = Unit::new(&[DEG_CELSIUS], &[]).unwrap();
        let b = Unit::new(&[RANKINE], &[]).unwrap();
        assert_relative_eq!(a.convert(1.0, &b).unwrap(), 1.8);
    }

    #[test]
    fn deg_fahrenheit_conversions() {
        let a = Unit::new(&[DEG_FAHRENHEIT], &[]).unwrap();
        let b = Unit::new(&[TEMP_CELSIUS], &[]).unwrap();
        assert!(a.convert(1.0, &b).is_err());
        let a = Unit::new(&[DEG_FAHRENHEIT], &[]).unwrap();
        let b = Unit::new(&[TEMP_FAHRENHEIT], &[]).unwrap();
        assert!(a.convert(1.0, &b).is_err());
        let a = Unit::new(&[DEG_FAHRENHEIT], &[]).unwrap();
        let b = Unit::new(&[DEG_CELSIUS], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 5.0 / 9.0);
        let a = Unit::new(&[DEG_FAHRENHEIT], &[]).unwrap();
        let b = Unit::new(&[DEG_FAHRENHEIT], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 1.0);
        let a = Unit::new(&[DEG_FAHRENHEIT], &[]).unwrap();
        let b = Unit::new(&[KELVIN], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 5.0 / 9.0);
        let a = Unit::new(&[DEG_FAHRENHEIT], &[]).unwrap();
        let b = Unit::new(&[RANKINE], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 1.0);
    }

    #[test]
    fn kelvin_conversions() {
        let a = Unit::new(&[KELVIN], &[]).unwrap();
        let b = Unit::new(&[TEMP_CELSIUS], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), -272.15);
        let a = Unit::new(&[KELVIN], &[]).unwrap();
        let b = Unit::new(&[TEMP_FAHRENHEIT], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), -457.87);
        let a = Unit::new(&[KELVIN], &[]).unwrap();
        let b = Unit::new(&[DEG_CELSIUS], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 1.0);
        let a = Unit::new(&[KELVIN], &[]).unwrap();
        let b = Unit::new(&[DEG_FAHRENHEIT], &[]).unwrap();
        assert_relative_eq!(a.convert(1.0, &b).unwrap(), 1.8);
        let a = Unit::new(&[KELVIN], &[]).unwrap();
        let b = Unit::new(&[KELVIN], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 1.0);
        let a = Unit::new(&[KELVIN], &[]).unwrap();
        let b = Unit::new(&[RANKINE], &[]).unwrap();
        assert_relative_eq!(a.convert(1.0, &b).unwrap(), 1.8);
    }

    #[test]
    fn deg_rankine_conversions() {
        let a = Unit::new(&[RANKINE], &[]).unwrap();
        let b = Unit::new(&[TEMP_CELSIUS], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), -273.15 + 5.0 / 9.0);
        let a = Unit::new(&[RANKINE], &[]).unwrap();
        let b = Unit::new(&[TEMP_FAHRENHEIT], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), -458.67);
        let a = Unit::new(&[RANKINE], &[]).unwrap();
        let b = Unit::new(&[DEG_CELSIUS], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 5.0 / 9.0);
        let a = Unit::new(&[RANKINE], &[]).unwrap();
        let b = Unit::new(&[DEG_FAHRENHEIT], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 1.0);
        let a = Unit::new(&[RANKINE], &[]).unwrap();
        let b = Unit::new(&[KELVIN], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 5.0 / 9.0);
        let a = Unit::new(&[RANKINE], &[]).unwrap();
        let b = Unit::new(&[RANKINE], &[]).unwrap();
        assert_eq!(a.convert(1.0, &b).unwrap(), 1.0);
    }

    #[test]
    fn temp_in_derived_unit() {
        let r = Unit::new(&[TEMP_CELSIUS, SECOND], &[]);
        assert!(r.is_err());
        let r = Unit::new(&[TEMP_CELSIUS], &[SECOND]);
        assert!(r.is_err());
        let r = Unit::new(&[], &[TEMP_CELSIUS]);
        assert!(r.is_err());
        let r = Unit::new(&[KELVIN, SECOND], &[]);
        assert!(r.is_ok());
        let r = Unit::new(&[KELVIN], &[SECOND]);
        assert!(r.is_ok());
        let r = Unit::new(&[], &[KELVIN]);
        assert!(r.is_ok());
    }

    #[test]
    fn inverse() {
        let u = Unit::new(&[KILOGRAM], &[SECOND, SECOND, AMPERE])
            .unwrap()
            .inverse()
            .unwrap();
        assert_eq!(
            u,
            Unit::new(&[SECOND, SECOND, AMPERE], &[KILOGRAM]).unwrap()
        );

        let u = Unit::new(&[TEMP_FAHRENHEIT], &[]).unwrap();
        assert!(u.inverse().is_err());
        let u = Unit::new(&[KELVIN], &[]).unwrap();
        assert!(u.inverse().is_ok());
    }
}
