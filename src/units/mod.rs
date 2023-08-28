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

//! Unit conversion.
#![allow(dead_code)]

use once_cell::sync::Lazy;

mod base;
mod number;
mod unit;

pub use base::{Base, PhysicalQuantity};
pub use number::Number;
pub use unit::Unit;

#[derive(Debug, PartialEq)]
pub enum Error {
    IncommensurableUnits(Option<Box<Unit>>, Option<Box<Unit>>),
    UninvertableUnits(Box<Unit>),
    NonzeroZeroPoint(Base),
    ExponentHasUnits,
    ExponentNotAnInteger,
    DegreeHasUnits,
    DegreeNotAnInteger,
    UnitNotDivisible,
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
pub static DAY: Base = Base::new("day", Time, 86400.0);
pub static HOUR: Base = Base::new("hr", Time, 3600.0);
pub static MINUTE: Base = Base::new("min", Time, 60.0);

// More lengths
pub static INCH: Base = Base::new("in", Length, 0.3048 / 12.0);
pub static FOOT: Base = Base::new("ft", Length, 0.3048);
pub static MILE: Base = Base::new("mi", Length, 1609.344);
pub static NAUTICAL_MILE: Base = Base::new("NM", Length, 1852.0);
pub static MIL: Base = Base::new("mil", Length, 0.000_304_8 / 12.0);
pub static YARD: Base = Base::new("yd", Length, 0.3048 * 3.0);

// More masses
pub static POUND_MASS: Base = Base::new("lb", Mass, 0.453_592_37);
pub static OUNCE: Base = Base::new("oz", Mass, 0.028_349_523_125);

// More temperatures
pub static RANKINE: Base = Base::new("R", Temperature, 5.0 / 9.0);
pub static DEG_CELSIUS: Base = Base::new("degC", Temperature, 1.0).without_zero();
pub static DEG_FAHRENHEIT: Base = Base::new("degF", Temperature, 5.0 / 9.0).without_zero();
pub static TEMP_CELSIUS: Base = Base::new("tempC", Temperature, 1.0).with_zero(-273.15);
pub static TEMP_FAHRENHEIT: Base = Base::new("tempF", Temperature, 5.0 / 9.0).with_zero(-459.67);

// More angles
pub static DEGREE: Base = Base::new("deg", Angle, std::f64::consts::PI / 180.0);

// Energy
pub static JOULE: Lazy<Unit> = Lazy::new(|| {
    Unit::new(&[KILOGRAM, METER, METER], &[SECOND, SECOND])
        .unwrap()
        .with_symbol("J")
});

// Force
pub static NEWTON: Lazy<Unit> = Lazy::new(|| {
    Unit::new(&[KILOGRAM, METER], &[SECOND, SECOND])
        .unwrap()
        .with_symbol("N")
});
pub static POUND_FORCE: Lazy<Unit> = Lazy::new(|| {
    Unit::new(&[POUND_MASS, METER], &[SECOND, SECOND])
        .unwrap()
        .with_constant(9.80665) // standard acceleration due to Earth's gravity
        .with_symbol("lbf")
});

// Power
pub static WATT: Lazy<Unit> = Lazy::new(|| (&*JOULE / SECOND).unwrap().with_symbol("W"));

// Pressure
pub static PASCAL: Lazy<Unit> = Lazy::new(|| {
    ((&*NEWTON / METER).unwrap() / METER)
        .unwrap()
        .with_symbol("Pa")
});
pub static PSI: Lazy<Unit> = Lazy::new(|| {
    ((&*POUND_FORCE / INCH).unwrap() / INCH)
        .unwrap()
        .with_symbol("psi")
});

// Electromagnetic
pub static VOLT: Lazy<Unit> = Lazy::new(|| {
    Unit::new(&[KILOGRAM, METER, METER], &[SECOND, SECOND, SECOND, AMPERE])
        .unwrap()
        .with_symbol("V")
});
pub static OHM: Lazy<Unit> = Lazy::new(|| {
    Unit::new(
        &[KILOGRAM, METER, METER],
        &[SECOND, SECOND, SECOND, AMPERE, AMPERE],
    )
    .unwrap()
    .with_symbol("ohm")
});
pub static FARAD: Lazy<Unit> = Lazy::new(|| {
    Unit::new(
        &[SECOND, SECOND, SECOND, SECOND, AMPERE, AMPERE],
        &[KILOGRAM, METER, METER],
    )
    .unwrap()
    .with_symbol("F")
});
pub static TESLA: Lazy<Unit> = Lazy::new(|| {
    Unit::new(&[KILOGRAM], &[SECOND, SECOND, AMPERE])
        .unwrap()
        .with_symbol("T")
});

// SI prefixes
pub static PETASECOND: Base = Base::new("Ps", Time, 1e15);
pub static TERASECOND: Base = Base::new("Ts", Time, 1e12);
pub static GIGASECOND: Base = Base::new("Gs", Time, 1e9);
pub static MEGASECOND: Base = Base::new("Ms", Time, 1e6);
pub static KILOSECOND: Base = Base::new("ks", Time, 1e3);
pub static MILLISECOND: Base = Base::new("ms", Time, 1e-3);
pub static MICROSECOND: Base = Base::new("us", Time, 1e-6);
pub static NANOSECOND: Base = Base::new("ns", Time, 1e-9);
pub static PICOSECOND: Base = Base::new("ps", Time, 1e-12);
pub static FEMTOSECOND: Base = Base::new("fs", Time, 1e-15);

pub static PETAMETER: Base = Base::new("Pm", Length, 1e15);
pub static TERAMETER: Base = Base::new("Tm", Length, 1e12);
pub static GIGAMETER: Base = Base::new("Gm", Length, 1e9);
pub static MEGAMETER: Base = Base::new("Mm", Length, 1e6);
pub static KILOMETER: Base = Base::new("km", Length, 1e3);
pub static CENTIMETER: Base = Base::new("cm", Length, 1e-2);
pub static MILLIMETER: Base = Base::new("mm", Length, 1e-3);
pub static MICROMETER: Base = Base::new("um", Length, 1e-6);
pub static NANOMETER: Base = Base::new("nm", Length, 1e-9);
pub static PICOMETER: Base = Base::new("pm", Length, 1e-12);
pub static FEMTOMETER: Base = Base::new("fm", Length, 1e-15);

pub static PETAGRAM: Base = Base::new("Pg", Mass, 1e15);
pub static TERAGRAM: Base = Base::new("Tg", Mass, 1e12);
pub static GIGAGRAM: Base = Base::new("Gg", Mass, 1e9);
pub static MEGAGRAM: Base = Base::new("Mg", Mass, 1e6);
pub static MILLIGRAM: Base = Base::new("mg", Mass, 1e-3);
pub static MICROGRAM: Base = Base::new("ug", Mass, 1e-6);
pub static NANOGRAM: Base = Base::new("ng", Mass, 1e-9);
pub static PICOGRAM: Base = Base::new("pg", Mass, 1e-12);
pub static FEMTOGRAM: Base = Base::new("fg", Mass, 1e-15);

pub static PETAAMPERE: Base = Base::new("PA", Current, 1e15);
pub static TERAAMPERE: Base = Base::new("TA", Current, 1e12);
pub static GIGAAMPERE: Base = Base::new("GA", Current, 1e9);
pub static MEGAAMPERE: Base = Base::new("MA", Current, 1e6);
pub static KILOAMPERE: Base = Base::new("kA", Current, 1e3);
pub static MILLIAMPERE: Base = Base::new("mA", Current, 1e-3);
pub static MICROAMPERE: Base = Base::new("uA", Current, 1e-6);
pub static NANOAMPERE: Base = Base::new("nA", Current, 1e-9);
pub static PICOAMPERE: Base = Base::new("pA", Current, 1e-12);
pub static FEMTOAMPERE: Base = Base::new("fA", Current, 1e-15);

pub static PETAKELVIN: Base = Base::new("PK", Temperature, 1e15);
pub static TERAKELVIN: Base = Base::new("TK", Temperature, 1e12);
pub static GIGAKELVIN: Base = Base::new("GK", Temperature, 1e9);
pub static MEGAKELVIN: Base = Base::new("MK", Temperature, 1e6);
pub static KILOKELVIN: Base = Base::new("kK", Temperature, 1e3);
pub static MILLIKELVIN: Base = Base::new("mK", Temperature, 1e-3);
pub static MICROKELVIN: Base = Base::new("uK", Temperature, 1e-6);
pub static NANOKELVIN: Base = Base::new("nK", Temperature, 1e-9);
pub static PICOKELVIN: Base = Base::new("pK", Temperature, 1e-12);
pub static FEMTOKELVIN: Base = Base::new("fK", Temperature, 1e-15);

pub static PETAMOLE: Base = Base::new("Pmol", AmountOfSubstance, 1e15);
pub static TERAMOLE: Base = Base::new("Tmol", AmountOfSubstance, 1e12);
pub static GIGAMOLE: Base = Base::new("Gmol", AmountOfSubstance, 1e9);
pub static MEGAMOLE: Base = Base::new("Mmol", AmountOfSubstance, 1e6);
pub static KILOMOLE: Base = Base::new("kmol", AmountOfSubstance, 1e3);
pub static MILLIMOLE: Base = Base::new("mmol", AmountOfSubstance, 1e-3);
pub static MICROMOLE: Base = Base::new("umol", AmountOfSubstance, 1e-6);
pub static NANOMOLE: Base = Base::new("nmol", AmountOfSubstance, 1e-9);
pub static PICOMOLE: Base = Base::new("pmol", AmountOfSubstance, 1e-12);
pub static FEMTOMOLE: Base = Base::new("fmol", AmountOfSubstance, 1e-15);

pub static PETACANDELA: Base = Base::new("Pcd", LuminousIntensity, 1e15);
pub static TERACANDELA: Base = Base::new("Tcd", LuminousIntensity, 1e12);
pub static GIGACANDELA: Base = Base::new("Gcd", LuminousIntensity, 1e9);
pub static MEGACANDELA: Base = Base::new("Mcd", LuminousIntensity, 1e6);
pub static KILOCANDELA: Base = Base::new("kcd", LuminousIntensity, 1e3);
pub static MILLICANDELA: Base = Base::new("mcd", LuminousIntensity, 1e-3);
pub static MICROCANDELA: Base = Base::new("ucd", LuminousIntensity, 1e-6);
pub static NANOCANDELA: Base = Base::new("ncd", LuminousIntensity, 1e-9);
pub static PICOCANDELA: Base = Base::new("pcd", LuminousIntensity, 1e-12);
pub static FEMTOCANDELA: Base = Base::new("fcd", LuminousIntensity, 1e-15);

pub static PETARADIAN: Base = Base::new("Prad", Angle, 1e15);
pub static TERARADIAN: Base = Base::new("Trad", Angle, 1e12);
pub static GIGARADIAN: Base = Base::new("Grad", Angle, 1e9);
pub static MEGARADIAN: Base = Base::new("Mrad", Angle, 1e6);
pub static KILORADIAN: Base = Base::new("krad", Angle, 1e3);
pub static MILLIRADIAN: Base = Base::new("mrad", Angle, 1e-3);
pub static MICRORADIAN: Base = Base::new("urad", Angle, 1e-6);
pub static NANORADIAN: Base = Base::new("nrad", Angle, 1e-9);
pub static PICORADIAN: Base = Base::new("prad", Angle, 1e-12);
pub static FEMTORADIAN: Base = Base::new("frad", Angle, 1e-15);

pub static PETAJOULE: Lazy<Unit> = Lazy::new(|| JOULE.with_constant(1e15).with_symbol("PJ"));
pub static TERAJOULE: Lazy<Unit> = Lazy::new(|| JOULE.with_constant(1e12).with_symbol("TJ"));
pub static GIGAJOULE: Lazy<Unit> = Lazy::new(|| JOULE.with_constant(1e9).with_symbol("GJ"));
pub static MEGAJOULE: Lazy<Unit> = Lazy::new(|| JOULE.with_constant(1e6).with_symbol("MJ"));
pub static KILOJOULE: Lazy<Unit> = Lazy::new(|| JOULE.with_constant(1e3).with_symbol("kJ"));
pub static MILLIJOULE: Lazy<Unit> = Lazy::new(|| JOULE.with_constant(1e-3).with_symbol("mJ"));
pub static MICROJOULE: Lazy<Unit> = Lazy::new(|| JOULE.with_constant(1e-6).with_symbol("uJ"));
pub static NANOJOULE: Lazy<Unit> = Lazy::new(|| JOULE.with_constant(1e-9).with_symbol("nJ"));
pub static PICOJOULE: Lazy<Unit> = Lazy::new(|| JOULE.with_constant(1e-12).with_symbol("pJ"));
pub static FEMTOJOULE: Lazy<Unit> = Lazy::new(|| JOULE.with_constant(1e-15).with_symbol("fJ"));

pub static PETANEWTON: Lazy<Unit> = Lazy::new(|| NEWTON.with_constant(1e15).with_symbol("PN"));
pub static TERANEWTON: Lazy<Unit> = Lazy::new(|| NEWTON.with_constant(1e12).with_symbol("TN"));
pub static GIGANEWTON: Lazy<Unit> = Lazy::new(|| NEWTON.with_constant(1e9).with_symbol("GN"));
pub static MEGANEWTON: Lazy<Unit> = Lazy::new(|| NEWTON.with_constant(1e6).with_symbol("MN"));
pub static KILONEWTON: Lazy<Unit> = Lazy::new(|| NEWTON.with_constant(1e3).with_symbol("kN"));
pub static MILLINEWTON: Lazy<Unit> = Lazy::new(|| NEWTON.with_constant(1e-3).with_symbol("mN"));
pub static MICRONEWTON: Lazy<Unit> = Lazy::new(|| NEWTON.with_constant(1e-6).with_symbol("uN"));
pub static NANONEWTON: Lazy<Unit> = Lazy::new(|| NEWTON.with_constant(1e-9).with_symbol("nN"));
pub static PICONEWTON: Lazy<Unit> = Lazy::new(|| NEWTON.with_constant(1e-12).with_symbol("pN"));
pub static FEMTONEWTON: Lazy<Unit> = Lazy::new(|| NEWTON.with_constant(1e-15).with_symbol("fN"));

pub static PETAWATT: Lazy<Unit> = Lazy::new(|| WATT.with_constant(1e15).with_symbol("PW"));
pub static TERAWATT: Lazy<Unit> = Lazy::new(|| WATT.with_constant(1e12).with_symbol("TW"));
pub static GIGAWATT: Lazy<Unit> = Lazy::new(|| WATT.with_constant(1e9).with_symbol("GW"));
pub static MEGAWATT: Lazy<Unit> = Lazy::new(|| WATT.with_constant(1e6).with_symbol("MW"));
pub static KILOWATT: Lazy<Unit> = Lazy::new(|| WATT.with_constant(1e3).with_symbol("kW"));
pub static MILLIWATT: Lazy<Unit> = Lazy::new(|| WATT.with_constant(1e-3).with_symbol("mW"));
pub static MICROWATT: Lazy<Unit> = Lazy::new(|| WATT.with_constant(1e-6).with_symbol("uW"));
pub static NANOWATT: Lazy<Unit> = Lazy::new(|| WATT.with_constant(1e-9).with_symbol("nW"));
pub static PICOWATT: Lazy<Unit> = Lazy::new(|| WATT.with_constant(1e-12).with_symbol("pW"));
pub static FEMTOWATT: Lazy<Unit> = Lazy::new(|| WATT.with_constant(1e-15).with_symbol("fW"));

pub static PETAPASCAL: Lazy<Unit> = Lazy::new(|| PASCAL.with_constant(1e15).with_symbol("PP"));
pub static TERAPASCAL: Lazy<Unit> = Lazy::new(|| PASCAL.with_constant(1e12).with_symbol("TP"));
pub static GIGAPASCAL: Lazy<Unit> = Lazy::new(|| PASCAL.with_constant(1e9).with_symbol("GP"));
pub static MEGAPASCAL: Lazy<Unit> = Lazy::new(|| PASCAL.with_constant(1e6).with_symbol("MP"));
pub static KILOPASCAL: Lazy<Unit> = Lazy::new(|| PASCAL.with_constant(1e3).with_symbol("kP"));
pub static MILLIPASCAL: Lazy<Unit> = Lazy::new(|| PASCAL.with_constant(1e-3).with_symbol("mP"));
pub static MICROPASCAL: Lazy<Unit> = Lazy::new(|| PASCAL.with_constant(1e-6).with_symbol("uP"));
pub static NANOPASCAL: Lazy<Unit> = Lazy::new(|| PASCAL.with_constant(1e-9).with_symbol("nP"));
pub static PICOPASCAL: Lazy<Unit> = Lazy::new(|| PASCAL.with_constant(1e-12).with_symbol("pP"));
pub static FEMTOPASCAL: Lazy<Unit> = Lazy::new(|| PASCAL.with_constant(1e-15).with_symbol("fP"));

pub static PETAVOLT: Lazy<Unit> = Lazy::new(|| VOLT.with_constant(1e15).with_symbol("PV"));
pub static TERAVOLT: Lazy<Unit> = Lazy::new(|| VOLT.with_constant(1e12).with_symbol("TV"));
pub static GIGAVOLT: Lazy<Unit> = Lazy::new(|| VOLT.with_constant(1e9).with_symbol("GV"));
pub static MEGAVOLT: Lazy<Unit> = Lazy::new(|| VOLT.with_constant(1e6).with_symbol("MV"));
pub static KILOVOLT: Lazy<Unit> = Lazy::new(|| VOLT.with_constant(1e3).with_symbol("kV"));
pub static MILLIVOLT: Lazy<Unit> = Lazy::new(|| VOLT.with_constant(1e-3).with_symbol("mV"));
pub static MICROVOLT: Lazy<Unit> = Lazy::new(|| VOLT.with_constant(1e-6).with_symbol("uV"));
pub static NANOVOLT: Lazy<Unit> = Lazy::new(|| VOLT.with_constant(1e-9).with_symbol("nV"));
pub static PICOVOLT: Lazy<Unit> = Lazy::new(|| VOLT.with_constant(1e-12).with_symbol("pV"));
pub static FEMTOVOLT: Lazy<Unit> = Lazy::new(|| VOLT.with_constant(1e-15).with_symbol("fV"));

pub static PETAOHM: Lazy<Unit> = Lazy::new(|| OHM.with_constant(1e15).with_symbol("Pohm"));
pub static TERAOHM: Lazy<Unit> = Lazy::new(|| OHM.with_constant(1e12).with_symbol("Tohm"));
pub static GIGAOHM: Lazy<Unit> = Lazy::new(|| OHM.with_constant(1e9).with_symbol("Gohm"));
pub static MEGAOHM: Lazy<Unit> = Lazy::new(|| OHM.with_constant(1e6).with_symbol("Mohm"));
pub static KILOOHM: Lazy<Unit> = Lazy::new(|| OHM.with_constant(1e3).with_symbol("kohm"));
pub static MILLIOHM: Lazy<Unit> = Lazy::new(|| OHM.with_constant(1e-3).with_symbol("mohm"));
pub static MICROOHM: Lazy<Unit> = Lazy::new(|| OHM.with_constant(1e-6).with_symbol("uohm"));
pub static NANOOHM: Lazy<Unit> = Lazy::new(|| OHM.with_constant(1e-9).with_symbol("nohm"));
pub static PICOOHM: Lazy<Unit> = Lazy::new(|| OHM.with_constant(1e-12).with_symbol("pohm"));
pub static FEMTOOHM: Lazy<Unit> = Lazy::new(|| OHM.with_constant(1e-15).with_symbol("fohm"));

pub static PETAFARAD: Lazy<Unit> = Lazy::new(|| FARAD.with_constant(1e15).with_symbol("PF"));
pub static TERAFARAD: Lazy<Unit> = Lazy::new(|| FARAD.with_constant(1e12).with_symbol("TF"));
pub static GIGAFARAD: Lazy<Unit> = Lazy::new(|| FARAD.with_constant(1e9).with_symbol("GF"));
pub static MEGAFARAD: Lazy<Unit> = Lazy::new(|| FARAD.with_constant(1e6).with_symbol("MF"));
pub static KILOFARAD: Lazy<Unit> = Lazy::new(|| FARAD.with_constant(1e3).with_symbol("kF"));
pub static MILLIFARAD: Lazy<Unit> = Lazy::new(|| FARAD.with_constant(1e-3).with_symbol("mF"));
pub static MICROFARAD: Lazy<Unit> = Lazy::new(|| FARAD.with_constant(1e-6).with_symbol("uF"));
pub static NANOFARAD: Lazy<Unit> = Lazy::new(|| FARAD.with_constant(1e-9).with_symbol("nF"));
pub static PICOFARAD: Lazy<Unit> = Lazy::new(|| FARAD.with_constant(1e-12).with_symbol("pF"));
pub static FEMTOFARAD: Lazy<Unit> = Lazy::new(|| FARAD.with_constant(1e-15).with_symbol("fF"));

pub static PETATESLA: Lazy<Unit> = Lazy::new(|| TESLA.with_constant(1e15).with_symbol("PT"));
pub static TERATESLA: Lazy<Unit> = Lazy::new(|| TESLA.with_constant(1e12).with_symbol("TT"));
pub static GIGATESLA: Lazy<Unit> = Lazy::new(|| TESLA.with_constant(1e9).with_symbol("GT"));
pub static MEGATESLA: Lazy<Unit> = Lazy::new(|| TESLA.with_constant(1e6).with_symbol("MT"));
pub static KILOTESLA: Lazy<Unit> = Lazy::new(|| TESLA.with_constant(1e3).with_symbol("kT"));
pub static MILLITESLA: Lazy<Unit> = Lazy::new(|| TESLA.with_constant(1e-3).with_symbol("mT"));
pub static MICROTESLA: Lazy<Unit> = Lazy::new(|| TESLA.with_constant(1e-6).with_symbol("uT"));
pub static NANOTESLA: Lazy<Unit> = Lazy::new(|| TESLA.with_constant(1e-9).with_symbol("nT"));
pub static PICOTESLA: Lazy<Unit> = Lazy::new(|| TESLA.with_constant(1e-12).with_symbol("pT"));
pub static FEMTOTESLA: Lazy<Unit> = Lazy::new(|| TESLA.with_constant(1e-15).with_symbol("fT"));
