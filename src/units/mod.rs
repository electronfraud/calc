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

//! Unit conversion.
#![allow(dead_code)]

use once_cell::sync::Lazy;

mod base;
mod number;
mod unit;

pub use base::{Base, PhysicalQuantity};
pub use number::Number;
pub use unit::Unit;

#[derive(Debug)]
pub enum Error {
    IncommensurableUnits(Option<Box<Unit>>, Option<Box<Unit>>),
}

#[allow(clippy::enum_glob_use)]
use PhysicalQuantity::*;

/// SI base unit for time
pub static SECOND: Base = Base::new("s", Time, 1.0);
/// SI base unit for length
pub static METER: Base = Base::new("m", Length, 1.0);
/// SI base unit for mass
pub static KILOGRAM: Base = Base::new("kg", Mass, 1.0);
/// SI base unit for electric current
pub static AMPERE: Base = Base::new("A", Current, 1.0);
/// SI base unit for thermodynamic temperature
pub static KELVIN: Base = Base::new("K", Temperature, 1.0);
/// SI base unit for amount of substance
pub static MOLE: Base = Base::new("mol", AmountOfSubstance, 1.0);
/// SI base unit for luminous intensity
pub static CANDELA: Base = Base::new("cd", LuminousIntensity, 1.0);
/// SI unit of angle
pub static RADIAN: Base = Base::new("rad", Angle, 1.0);

// More times
pub static HOUR: Base = Base::new("hr", Time, 3600.0);
pub static MINUTE: Base = Base::new("min", Time, 60.0);

// More lengths
pub static CENTIMETER: Base = Base::new("cm", Length, 0.01);
pub static MILLIMETER: Base = Base::new("mm", Length, 0.001);

pub static INCH: Base = Base::new("in", Length, 0.3048 / 12.0);
pub static FOOT: Base = Base::new("ft", Length, 0.3048);
pub static MILE: Base = Base::new("mi", Length, 1609.344);
pub static NAUTICAL_MILE: Base = Base::new("NM", Length, 1852.0);

// More temperatures
pub static CELSIUS: Base = Base::new("degC", Temperature, 1.0).with_zero(-273.15);
pub static FAHRENHEIT: Base = Base::new("degF", Temperature, 5.0 / 9.0).with_zero(-459.67);
pub static RANKINE: Base = Base::new("R", Temperature, 5.0 / 9.0);

// More angles
pub static DEGREE: Base = Base::new("deg", Angle, std::f64::consts::PI / 180.0);

// Energy
pub static JOULE: Lazy<Unit> = Lazy::new(|| Unit {
    symbol: Some(String::from("J")),
    numer: vec![&KILOGRAM, &METER, &METER],
    denom: vec![&SECOND, &SECOND],
});

// Force
pub static NEWTON: Lazy<Unit> = Lazy::new(|| Unit {
    symbol: Some(String::from("N")),
    numer: vec![&KILOGRAM, &METER],
    denom: vec![&SECOND, &SECOND],
});

// Power
pub static WATT: Lazy<Unit> = Lazy::new(|| (&*JOULE / &SECOND).with_symbol("W"));
