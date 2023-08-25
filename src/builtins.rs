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

//! Built-in functions and constants.

use std::collections::HashMap;

use crate::{commit, pop_as_f, pop_as_ff, pop_as_fu, pop_as_i, pop_as_ii, popn, popnn};
use crate::{
    integer, stack,
    stack::Stack,
    units,
    units::{Number, Unit, JOULE, METER, RADIAN, SECOND},
};

/// An error that occurred while executing a builtin.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// A stack error that occurred while executing a builtin.
    Stack(stack::Error),
    /// A units error that occurred while executing a builtin.
    Units(units::Error),
    /// A number was expected to have a unit but was dimensionless.
    MissingUnit,
    /// A number was expected to be dimensionless but had a unit.
    NotDimensionless,
    /// A number was expected to be non-negative but was negative.
    NotNonNegative,
    /// A number was expected to be whole but had a fractional part.
    NotWhole,
}

/// Enables the `?` operator inside implementations of builtins.
impl From<stack::Error> for Error {
    fn from(e: stack::Error) -> Error {
        Error::Stack(e)
    }
}

/// Enables the `?` operator inside implementations of builtins.
impl From<units::Error> for Error {
    fn from(e: units::Error) -> Error {
        Error::Units(e)
    }
}

/// The return type of a builtin.
type Result = std::result::Result<(), Error>;

/// A function that implements a builtin.
pub type Builtin = fn(&mut Stack) -> Result;

/// A table of builtin function names and their implementations.
pub type Table = HashMap<&'static str, Builtin>;

/// `( a b -- a+b )` Pops two items, adds them, and pushes the result.
///
/// # Errors
///
/// An error occurs if:
/// - there are fewer than two items on the stack;
/// - the items are not numbers; or,
/// - the items have incommensurable units.
#[allow(clippy::missing_panics_doc)]
pub fn builtin_add(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let (a, b) = popnn!(tx)?;
    match (a, b) {
        (stack::Item::Float(a), stack::Item::Float(b)) => tx.pushf((&a + &b)?),
        (stack::Item::Float(a), stack::Item::Integer(b)) => tx.pushf((&a + &b.as_units_number())?),
        (stack::Item::Integer(a), stack::Item::Float(b)) => tx.pushf((&a + &b)?),
        (stack::Item::Integer(a), stack::Item::Integer(b)) => tx.pushi(&a + &b),
        _ => panic!("invariant wasn't"),
    }
    commit!(tx)
}

/// `( a b -- a-b )` Pops two items, subtracts the upper item from the lower
/// item, and pushes the result.
///
/// # Errors
///
/// An error occurs if:
/// - there are fewer than two items on the stack;
/// - the items are not numbers; or,
/// - the items have incommensurable units.
#[allow(clippy::missing_panics_doc)]
pub fn builtin_sub(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let (a, b) = popnn!(tx)?;
    match (a, b) {
        (stack::Item::Float(a), stack::Item::Float(b)) => tx.pushf((&a - &b)?),
        (stack::Item::Float(a), stack::Item::Integer(b)) => tx.pushf((&a - &b.as_units_number())?),
        (stack::Item::Integer(a), stack::Item::Float(b)) => tx.pushf((&a - &b)?),
        (stack::Item::Integer(a), stack::Item::Integer(b)) => tx.pushi(&a - &b),
        _ => panic!("invariant wasn't"),
    }
    commit!(tx)
}

/// `( a b -- a*b )` Pops two items, multiplies them, and pushes the result.
///
/// The following combinations of operands are accepted:
/// - two numbers
/// - two units
/// - `a` is a number and `b` is a unit
///
/// Multiplying two units produces a new derived unit. Multiplying a number by
/// a unit is equivalent to multiplying by one of that unit.
///
/// # Errors
///
/// An error occurs if:
/// - there are fewer than two items on the stack;
/// - the items are not two numbers;
/// - the items are not two units; or,
/// - the items are not a number `a` and a unit `b`.
/// - the operation would result in a nonsensical temperature unit.
pub fn builtin_mul(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let items = tx.pop2()?;
    match items {
        (stack::Item::Float(a), stack::Item::Float(b)) => tx.pushf((&a * &b)?),
        (stack::Item::Float(a), stack::Item::Integer(b)) => tx.pushf((&a * &b.as_units_number())?),
        (stack::Item::Integer(a), stack::Item::Float(b)) => tx.pushf((&a * &b)?),
        (stack::Item::Integer(a), stack::Item::Integer(b)) => tx.pushi(&a * &b),
        (stack::Item::Unit(a), stack::Item::Unit(b)) => tx.pushu((&a * &b)?),
        (stack::Item::Float(a), stack::Item::Unit(b)) => tx.pushf((&a * &b)?),
        (stack::Item::Integer(a), stack::Item::Unit(b)) => tx.pushf((&a * &b)?),
        _ => return Err(stack::Error::TypeMismatch.into()),
    };
    commit!(tx)
}

/// `( a b -- a/b )` Pops two items, divides them, and pushes the result.
///
/// The following combinations of operands are accepted:
/// - two numbers
/// - two units
/// - `a` is a number and `b` is a unit
///
/// Dividing two units produces a new derived unit. Dividing a number by a unit
/// is equivalent to multiplying the number by the unit's inverse.
///
/// # Errors
///
/// An error occurs if:
/// - there are fewer than two items on the stack;
/// - the items are not two numbers;
/// - the items are not two units;
/// - the items are not a number `a` and a unit `b`;
/// - the operation would result in a nonsensical temperature unit.
pub fn builtin_div(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let items = tx.pop2()?;
    match items {
        (stack::Item::Float(a), stack::Item::Float(b)) => tx.pushf((&a / &b)?),
        (stack::Item::Float(a), stack::Item::Integer(b)) => tx.pushf((&a / &b.as_units_number())?),
        (stack::Item::Integer(a), stack::Item::Float(b)) => tx.pushf((&a / &b)?),
        (stack::Item::Integer(a), stack::Item::Integer(b)) => tx.pushf(&a / &b),
        (stack::Item::Unit(a), stack::Item::Unit(b)) => tx.pushu((&a / &b)?),
        (stack::Item::Float(a), stack::Item::Unit(b)) => tx.pushf((&a / &b)?),
        (stack::Item::Integer(a), stack::Item::Unit(b)) => tx.pushf((&a / &b)?),
        _ => return Err(stack::Error::TypeMismatch.into()),
    };
    commit!(tx)
}

/// `( a b -- a**b )` Raises `a` to the power of `b`.
///
/// The following combinations of operands are accepted:
/// - two dimensionless numbers
/// - `a` is a number with units and `b` is a dimensionless integer
///
/// Raising a number with units to a large power is not recommended.
///
/// # Errors
///
/// Returns an error if:
/// - there are fewer than two items on the stack;
/// - the operation would result in a nonsensical temperature unit; or,
/// - the items are not one of the accepted combinations described above.
pub fn builtin_pow(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let (a, b) = pop_as_ff!(tx)?;
    tx.pushf(a.pow(&b)?);
    commit!(tx)
}

/// `( a -- e**a )` Raises e to the power of `a`.
///
/// # Errors
///
/// Returns an error if:
/// - the stack is empty; or,
/// - the exponent is not dimensionless.
pub fn builtin_exp(stack: &mut Stack) -> Result {
    // This is functionally identical to `e swap **`, which makes it a prime
    // candidate for pulling out into a library once that's possible.
    let mut tx = stack.begin();
    let x = pop_as_f!(tx)?;
    tx.pushf(units::Number::new(std::f64::consts::E).pow(&x)?);
    commit!(tx)
}

/// `( a -- a**1/2 )` Finds the square root of `a`.
///
/// # Errors
///
/// Returns an error if:
/// - the stack is empty;
/// - `a` has units that don't have an integral square root.
pub fn builtin_sqrt(stack: &mut Stack) -> Result {
    // Another library candidate: `2 /**`
    let mut tx = stack.begin();
    let a = pop_as_f!(tx)?;
    tx.pushf(a.root(&units::Number::new(2.0))?);
    commit!(tx)
}

/// `( a -- a**1/3 )` Finds the cube root of `a`.
///
/// # Errors
///
/// Returns an error if:
/// - the stack is empty;
/// - `a` has units that don't have an integral cube root.
pub fn builtin_cbrt(stack: &mut Stack) -> Result {
    // Library candidate: `3 /**`
    let mut tx = stack.begin();
    let a = pop_as_f!(tx)?;
    tx.pushf(a.root(&units::Number::new(3.0))?);
    commit!(tx)
}

/// `( a b -- a**1/b )` Finds the `b`th root of `a`.
///
/// # Errors
///
/// Returns an error if:
/// - there are fewer than two items on the stack;
/// - `b` has units;
/// - `a` has units and `b` is not whole; or,
/// - `a` has units that don't have an integral `b`th root.
pub fn builtin_root(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let (a, b) = pop_as_ff!(tx)?;
    tx.pushf(a.root(&b)?);
    commit!(tx)
}

/// Macro for creating a trigonometric function builtin.
macro_rules! trig {
    ($name: ident, $fn: ident) => {
        /// `(a -- b)` Computes a trigonometric function.
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// - the stack is empty;
        /// - the item on top of the stack is not a number; or,
        /// - the number does not have units measuring an angle.
        pub fn $name(stack: &mut Stack) -> Result {
            let mut tx = stack.begin();
            let n = pop_as_f!(tx)?;

            if let Some(u) = n.unit {
                let n = u.convert(n.value, &RADIAN.as_unit())?;
                tx.pushx(n.$fn());
                commit!(tx)
            } else {
                Err(Error::MissingUnit)
            }
        }
    };
}

trig!(builtin_sin, sin);
trig!(builtin_cos, cos);
trig!(builtin_tan, tan);

/// Macro for creating an inverse trigonometric function.
macro_rules! inverse_trig {
    ($name: ident, $fn: ident) => {
        /// `(a -- b)` Computes an inverse trigonometric function.
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// - the stack is empty;
        /// - the item on top of the stack is not a number; or,
        /// - the number is not dimensionless.
        pub fn $name(stack: &mut Stack) -> Result {
            let mut tx = stack.begin();
            let n = pop_as_f!(tx)?;

            if n.unit.is_none() {
                tx.pushf(Number::new(n.value.$fn()).with_unit(RADIAN.as_unit()));
                commit!(tx)
            } else {
                Err(Error::NotDimensionless)
            }
        }
    };
}

inverse_trig!(builtin_asin, asin);
inverse_trig!(builtin_acos, acos);
inverse_trig!(builtin_atan, atan);

/// `( ... -- )` Pops everything from the stack.
///
/// # Errors
///
/// Never returns an error.
pub fn builtin_clear(stack: &mut Stack) -> Result {
    stack.clear();
    Ok(())
}

/// `( a -- a a )` Duplicates the item on top of the stack.
///
/// # Errors
///
/// An error occurs if the stack is empty.
pub fn builtin_dup(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let a = tx.pop()?;
    tx.push(a.clone());
    tx.push(a);
    commit!(tx)
}

/// `( [n u] -- n )` Makes a number dimensionless.
///
/// # Errors
///
/// An error occurs if:
/// - the stack is empty; or,
/// - the item on top of the stack is not a number.
pub fn builtin_drop(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    match tx.pop()? {
        stack::Item::Float(x) => {
            tx.pushx(x.value);
            commit!(tx)
        }
        stack::Item::Integer(_) => Ok(()),
        stack::Item::Unit(_) => Err(Error::Stack(stack::Error::TypeMismatch)),
    }
}

/// `( [n u1] u2 -- [n u2] )` Converts a number into different units.
///
/// # Errors
///
/// An error occurs if:
/// - there are fewer than two items on the stack;
/// - the items are not a number and a unit; or,
/// - the number has units that are incommensurable with `u`.
pub fn builtin_into(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let (a, u) = pop_as_fu!(tx)?;
    if let Some(a_unit) = a.unit {
        let b = a_unit.convert(a.value, &u)?;
        tx.pushf(Number::new(b).with_unit(u));
    } else {
        tx.pushf(a.with_unit(u));
    }
    commit!(tx)
}

macro_rules! bitwise {
    ($name: ident, $op: tt) => {
        /// `( a b -- c )` Computes a bitwise function of two integers.
        ///
        /// # Errors
        ///
        /// An error occurs if:
        /// - there are fewer than two items on the stack; or,
        /// - the items are not integers.
        pub fn $name(stack: &mut Stack) -> Result {
            let mut tx = stack.begin();
            let (a, b) = pop_as_ii!(tx)?;
            tx.pushi(integer::Integer::new(a.value $op b.value, a.repr));
            commit!(tx)
        }
    };
}

bitwise!(builtin_bitwise_and, &);
bitwise!(builtin_bitwise_or, |);
bitwise!(builtin_bitwise_xor, ^);

/// `( a -- ~a )` Computes the bitwise complement of an integer.
///
/// # Errors
///
/// An error occurs if:
/// - the stack is empty; or,
/// - the item on top of the stack is not an integer.
pub fn builtin_bitwise_complement(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let x = pop_as_i!(tx)?;
    tx.pushi(integer::Integer::new(!x.value, x.repr));
    commit!(tx)
}

macro_rules! binrepr {
    ($name: ident, $repr: expr) => {
        /// `( a -- a )` Changes the representation of an integer.
        ///
        /// # Errors
        /// An error occurs if:
        /// - the stack is empty; or,
        /// - the item on top of the stack is not an integer.
        pub fn $name(stack: &mut Stack) -> Result {
            let mut tx = stack.begin();
            let x = pop_as_i!(tx)?;
            tx.pushi(x.with_repr($repr));
            commit!(tx)
        }
    };
}

binrepr!(builtin_bin, integer::Representation::Binary);
binrepr!(builtin_dec, integer::Representation::Decimal);
binrepr!(builtin_oct, integer::Representation::Octal);
binrepr!(builtin_hex, integer::Representation::Hexadecimal);

/// `( a b -- [a & (1<<b)] )` Sets the bit in `a` at index `b`. The least
/// significant bit is index zero.
///
/// # Errors
///
/// An error occurs if:
/// - there are fewer than two items on the stack;
/// - `a` is not an integer; or,
/// - `b` is not a non-negative integer.
pub fn builtin_bset(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let (a, b) = pop_as_ii!(tx)?;
    if b.value >= 0 {
        tx.pushi(integer::Integer::new(a.value & (1 << b.value), a.repr));
        commit!(tx)
    } else {
        Err(Error::NotNonNegative)
    }
}

/// `( a b -- [a & ~(1<<b)] )` Clears the bit in `a` at index `b`. The least
/// significant bit is index zero.
///
/// # Errors
///
/// An error occurs if:
/// - there are fewer than two items on the stack;
/// - `a` is not an integer; or,
/// - `b` is not a non-negative integer.
pub fn builtin_bclr(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let (a, b) = pop_as_ii!(tx)?;
    if b.value >= 0 {
        tx.pushi(integer::Integer::new(a.value & !(1 << b.value), a.repr));
        commit!(tx)
    } else {
        Err(Error::NotNonNegative)
    }
}

/// `( a b -- a [(a >> b) & 1] )` Pushes the bit in `a` at index `b`. The least
/// significant bit is index zero.
///
/// # Errors
///
/// An error occurs if:
/// - there are fewer than two items on the stack;
/// - `a` is not an integer; or,
/// - `b` is not a non-negative integer.
pub fn builtin_bget(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let (a, b) = pop_as_ii!(tx)?;
    if b.value >= 0 {
        let bit = (a.value >> b.value) & 1;
        tx.pushi(a);
        tx.pushi(integer::Integer::bin(bit));
        commit!(tx)
    } else {
        Err(Error::NotNonNegative)
    }
}

/// `( ... a1 ... aN N -- a1 ... aN )` Removes everything from the stack except
/// the topmost `N` items.
///
/// # Errors
///
/// Returns an error if:
/// - the stack has fewer than N+1 items; or,
/// - the item on top of the stack is not a dimensionless, whole, non-negative
///   number.
pub fn builtin_keep(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let n = pop_as_i!(tx)?;
    if n.value < 0 {
        return Err(Error::NotNonNegative);
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    tx.keep(n.value as usize)?;
    commit!(tx)
}

/// `( a -- )` Pops an item off the stack.
///
/// # Errors
///
/// Returns an error if the stack is empty.
pub fn builtin_pop(stack: &mut Stack) -> Result {
    stack
        .pop()
        .map(|_| ())
        .ok_or(Error::Stack(stack::Error::Underflow))
}

/// `( a b -- b a )` Swaps the top two items on the stack.
///
/// # Errors
///
/// Returns an error if the stack has fewer than two items.
pub fn builtin_swap(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let (a, b) = tx.pop2()?;
    tx.push(b);
    tx.push(a);
    commit!(tx)
}

/// Builtin for words that are units.
///
/// If the item on top of the stack is a dimensionless number, that number is
/// assigned the unit `u`. Otherwise, `u` is pushed onto the stack.
#[allow(clippy::missing_panics_doc)]
pub fn builtin_unit(u: &Unit, stack: &mut Stack) {
    let mut tx = stack.begin();
    if let Ok(x) = popn!(tx) {
        match x {
            stack::Item::Float(x) => {
                if x.is_dimensionless() {
                    tx.pushf(x.with_unit(u.clone()));
                    return tx.commit();
                }
            }
            stack::Item::Integer(x) => {
                tx.pushf(x.as_units_number().with_unit(u.clone()));
                return tx.commit();
            }
            stack::Item::Unit(_) => panic!("invariant wasn't"),
        }
    }
    stack.pushu(u.clone());
}

/// Creates a builtin for an anonymous `Unit` (a unit without a symbol) that
/// pushes the unit.
macro_rules! anonunit {
    ($u:expr) => {
        |stack| {
            builtin_unit($u, stack);
            Ok(())
        }
    };
}

/// Creates a builtin for a `Base` that pushes a unit.
macro_rules! base {
    ($b:expr) => {
        ($b.symbol, anonunit!(&Unit::new(&[&$b], &[]).unwrap()))
    };
}

/// Creates a builtin for a named `Unit` (a unit with a symbol) that pushes the
/// unit.
macro_rules! unit {
    ($u:expr) => {
        ($u.symbol.as_ref().unwrap().as_str(), anonunit!($u))
    };
}

/// Creates a builtin for a dimensionless constant that pushes the constant.
macro_rules! constx {
    ($value:expr) => {
        |stack| {
            stack.pushx($value);
            Ok(())
        }
    };
}

/// Creates a builtin for a constant with units that pushes the constant.
macro_rules! constf {
    ($value:expr, $unit:expr) => {
        |stack| {
            stack.pushf(Number::new($value).with_unit(($unit).unwrap()));
            Ok(())
        }
    };
}

/// Returns a table of builtin names and the functions that implement them.
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn table() -> Table {
    HashMap::from([
        // Constants
        ("c", constf!(299_792_458.0, &METER / &SECOND) as Builtin),
        ("e", constx!(std::f64::consts::E)),
        ("h", constf!(6.626_070_15e-34, &*JOULE * &SECOND)),
        ("hbar", constf!(1.054_571_817e-34, &*JOULE * &SECOND)),
        ("pi", constx!(std::f64::consts::PI)),
        // Arithmetic
        ("+", builtin_add),
        ("-", builtin_sub),
        ("*", builtin_mul),
        ("/", builtin_div),
        ("**", builtin_pow),
        ("exp", builtin_exp),
        ("sqrt", builtin_sqrt),
        ("cbrt", builtin_cbrt),
        ("/**", builtin_root),
        // Trigonometric
        ("sin", builtin_sin),
        ("cos", builtin_cos),
        ("tan", builtin_tan),
        ("asin", builtin_asin),
        ("acos", builtin_acos),
        ("atan", builtin_atan),
        // Unit Conversion
        ("drop", builtin_drop),
        ("into", builtin_into),
        // Bitwise Operations
        ("&", builtin_bitwise_and),
        ("|", builtin_bitwise_or),
        ("^", builtin_bitwise_xor),
        ("~", builtin_bitwise_complement),
        ("bin", builtin_bin),
        ("oct", builtin_oct),
        ("dec", builtin_dec),
        ("hex", builtin_hex),
        ("bset", builtin_bset),
        ("bclr", builtin_bclr),
        ("bget", builtin_bget),
        // Stack Manipulation
        ("clear", builtin_clear),
        ("dup", builtin_dup),
        ("keep", builtin_keep),
        ("pop", builtin_pop),
        ("swap", builtin_swap),
        // Units
        base!(units::SECOND),
        base!(units::METER),
        base!(units::KILOGRAM),
        base!(units::AMPERE),
        base!(units::KELVIN),
        base!(units::MOLE),
        base!(units::CANDELA),
        base!(units::RADIAN),
        base!(units::HOUR),
        base!(units::FOOT),
        base!(units::MILE),
        base!(units::NAUTICAL_MILE),
        base!(units::DEG_CELSIUS),
        base!(units::DEG_FAHRENHEIT),
        base!(units::RANKINE),
        base!(units::TEMP_CELSIUS),
        base!(units::TEMP_FAHRENHEIT),
        base!(units::DEGREE),
        base!(units::INCH),
        base!(units::MILLIMETER),
        base!(units::CENTIMETER),
        base!(units::MINUTE),
        base!(units::DAY),
        base!(units::MIL),
        unit!(&*units::NEWTON),
        unit!(&*units::JOULE),
        unit!(&*units::WATT),
    ])
}
