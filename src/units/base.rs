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

use super::Unit;

/// A physical property measured by a unit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PhysicalQuantity {
    Time,
    Length,
    Mass,
    Current,
    Temperature,
    AmountOfSubstance,
    LuminousIntensity,
    Angle,
}

/// The number of different physical quantities.
pub const NUM_PHYSICAL_QUANTITIES: usize = 8;

/// A unit expressed in terms of one and only one physical quantity.
///
/// `Base` is not used by itself to perform unit conversions. Instead, it is
/// used to build `Unit`s, which do perform conversions.
#[derive(Clone, Debug)]
pub struct Base {
    /// Symbolic representation of the unit, e.g. "m" for meters.
    pub symbol: &'static str,
    /// Physical quantity measured by the unit.
    pub physq: PhysicalQuantity,
    /// Conversion factor to get SI units from this unit. In other words, how
    /// many of the corresponding SI base unit are equal to one of this unit.
    pub factor: f64,
    /// The value in this unit that equals zero in the corresponding SI base
    /// unit. For example, for Celsius this field is -273.15.
    pub zero: f64,
}

impl Base {
    /// Convenience function for creating a `Base` unit in which zero is equal
    /// to zero in the corresponding SI base unit.
    #[must_use]
    pub const fn new(symbol: &'static str, physq: PhysicalQuantity, factor: f64) -> Self {
        Self {
            symbol,
            physq,
            factor,
            zero: 0.0,
        }
    }

    /// Returns a new `Base` unit identical to this unit except that zero in
    /// the corresponding SI base unit equals `z` in this unit.
    #[must_use]
    pub const fn with_zero(&self, z: f64) -> Self {
        Self {
            symbol: self.symbol,
            physq: self.physq,
            factor: self.factor,
            zero: z,
        }
    }
}

impl PartialEq<Base> for Base {
    /// A `Base` equals another `Base` if they measure the same physical
    /// quantity, have the same factor, and have the same zero point.
    fn eq(&self, other: &Self) -> bool {
        self.physq == other.physq && self.factor == other.factor && self.zero == other.zero
    }
}

impl std::ops::Mul<&'static Base> for &'static Base {
    type Output = Unit;

    /// Produces a derived unit `self`⋅`other`.
    fn mul(self, other: Self) -> Unit {
        Unit::new(&[self, other], &[])
    }
}

impl std::ops::Mul<Unit> for &'static Base {
    type Output = Unit;

    /// Produces a derived unit `self`⋅`other`.
    fn mul(self, other: Unit) -> Unit {
        let mut numer = vec![self];
        numer.extend(other.numer);
        Unit::new(numer.as_slice(), &other.denom)
    }
}

impl std::ops::Div<&'static Base> for &'static Base {
    type Output = Unit;

    /// Produces a derived unit `self`⋅`other`⁻¹.
    fn div(self, other: Self) -> Unit {
        Unit::new(&[self], &[other])
    }
}

impl std::ops::Div<Unit> for &'static Base {
    type Output = Unit;

    /// Produces a derived unit `self`⋅`other`⁻¹.
    fn div(self, other: Unit) -> Unit {
        let mut numer = vec![self];
        numer.extend(other.denom);
        Unit::new(numer.as_slice(), &other.numer)
    }
}

#[cfg(test)]
mod tests {
    use crate::units::{KILOGRAM, METER, SECOND};

    #[test]
    fn base_multiplied_by_base() {
        let m_kg = &METER * &KILOGRAM;
        assert_eq!(m_kg.numer, vec![&METER, &KILOGRAM]);
        assert_eq!(m_kg.denom.len(), 0);
    }

    #[test]
    fn base_multiplied_by_unit() {
        let m_per_s = &METER / &SECOND;
        let kg_m_per_s = &KILOGRAM * m_per_s;
        assert_eq!(kg_m_per_s.numer, vec![&KILOGRAM, &METER]);
        assert_eq!(kg_m_per_s.denom, vec![&SECOND]);
    }

    #[test]
    fn base_divided_by_base() {
        let m_per_s = &METER / &SECOND;
        assert_eq!(m_per_s.numer, vec![&METER]);
        assert_eq!(m_per_s.denom, vec![&SECOND]);
    }

    #[test]
    fn base_divided_by_unit() {
        let m_per_s = &METER / &SECOND;
        let kg_s_per_m = &KILOGRAM / m_per_s;
        assert_eq!(kg_s_per_m.numer, vec![&KILOGRAM, &SECOND]);
        assert_eq!(kg_s_per_m.denom, vec![&METER]);
    }
}
