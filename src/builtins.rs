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

//! Built-in words.

use std::collections::HashMap;

use crate::{
    binary, stack,
    stack::Stack,
    units,
    units::{Number, Unit, RADIAN},
};
use crate::{commit, popb, popbb, popn, popnn, popnu};

pub enum Error {
    /// A stack error that occurred while executing a builtin.
    Stack(stack::Error),
    /// A units error that occurred while executing a builtin.
    Units(units::Error),
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

/// `( a b -- a+b )` Pops two items, adds them, and pushes the result.
///
/// # Errors
///
/// An error occurs if:
/// - there are fewer than two items on the stack;
/// - the items are not numbers; or,
/// - the items have incommensurable units.
pub fn builtin_add(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let (a, b) = popnn!(tx)?;
    tx.pushn((&a + &b)?);
    commit!(tx)
}

/// `( a b -- a-b )` Pops two items, subtracts the topmost item from the other
/// item, and pushes the result.
///
/// # Errors
///
/// An error occurs if:
/// - there are fewer than two items on the stack;
/// - the items are not numbers; or,
/// - the items have incommensurable units.
pub fn builtin_sub(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let (a, b) = popnn!(tx)?;
    tx.pushn((&a - &b)?);
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
///
/// If there is an error, the stack is unchanged; the operands are not popped.
pub fn builtin_mul(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let items = tx.pop2()?;
    match items {
        (stack::Item::Number(a), stack::Item::Number(b)) => tx.pushn((&a * &b)?),
        (stack::Item::Unit(a), stack::Item::Unit(b)) => tx.pushu((&a * &b)?),
        (stack::Item::Number(a), stack::Item::Unit(b)) => tx.pushn((&a * &b)?),
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
/// - the division would result in a nonsensical temperature unit.
///
/// If there is an error, the stack is unchanged; the operands are not popped.
pub fn builtin_div(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let items = tx.pop2()?;
    match items {
        (stack::Item::Number(a), stack::Item::Number(b)) => tx.pushn((&a / &b)?),
        (stack::Item::Unit(a), stack::Item::Unit(b)) => tx.pushu((&a / &b)?),
        (stack::Item::Number(a), stack::Item::Unit(b)) => tx.pushn((&a / &b)?),
        _ => return Err(stack::Error::TypeMismatch.into()),
    };
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
            let n = popn!(tx)?;

            if let Some(u) = n.unit {
                let n = u.convert(n.value, &RADIAN.as_unit())?;
                tx.pushv(n.$fn());
                commit!(tx)
            } else {
                Err(Error::Stack(stack::Error::TypeMismatch))
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
            let n = popn!(tx)?;

            if n.unit.is_none() {
                tx.pushn(Number::new(n.value.$fn()).with_unit(RADIAN.as_unit()));
                commit!(tx)
            } else {
                Err(Error::Stack(stack::Error::TypeMismatch))
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
    let n = popn!(tx)?;
    tx.pushv(n.value);
    commit!(tx)
}

/// `( [n u1] u2 -- [n u2] )` Converts a number into different units.
///
/// # Errors
///
/// An error occurs if:
/// - there are fewer than two items on the stack;
/// - the items are not a number and a unit; or,
/// - the number has units that are incommensurable with `u`.
///
/// If there is an error, the stack is unchanged; the operands are not popped.
pub fn builtin_into(stack: &mut Stack) -> Result {
    let mut tx = stack.begin();
    let (a, u) = popnu!(tx)?;
    if let Some(a_unit) = a.unit {
        let b = a_unit.convert(a.value, &u)?;
        tx.pushn(Number::new(b).with_unit(u));
    } else {
        tx.pushn(a.with_unit(u));
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
            let (a, b) = popbb!(tx)?;
            tx.pushb(binary::Integer::new(a.value $op b.value, a.repr));
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
    let b = popb!(tx)?;
    tx.pushb(binary::Integer::new(!b.value, b.repr));
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
            let b = popb!(tx)?;
            tx.pushb(b.with_repr($repr));
            commit!(tx)
        }
    };
}

binrepr!(builtin_bin, binary::Representation::Binary);
binrepr!(builtin_dec, binary::Representation::Decimal);
binrepr!(builtin_oct, binary::Representation::Octal);
binrepr!(builtin_hex, binary::Representation::Hexadecimal);

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
    let n = popn!(tx)?;
    if !n.is_dimensionless() || n.value.fract() != 0.0 || n.value < 0.0 {
        return Err(Error::Stack(stack::Error::TypeMismatch));
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
#[allow(clippy::missing_panics_doc)]
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
pub fn builtin_unit(u: &Unit, stack: &mut Stack) {
    let mut tx = stack.begin();
    if let Ok(n) = popn!(tx) {
        if n.is_dimensionless() {
            tx.pushn(n.with_unit(u.clone()));
            return tx.commit();
        }
    }
    stack.pushu(u.clone());
}

/// Creates a `Builtin` for a `Base`.
macro_rules! base {
    ($b:expr) => {
        ($b.symbol, anonunit!(&Unit::new(&[&$b], &[]).unwrap()))
    };
}

/// Creates a `Builtin` for an anonymous `Unit`.
macro_rules! anonunit {
    ($u:expr) => {
        |stack| {
            builtin_unit($u, stack);
            Ok(())
        }
    };
}

/// Creates a `Builtin` for a `Unit`.
macro_rules! unit {
    ($u:expr) => {
        ($u.symbol.as_ref().unwrap().as_str(), anonunit!($u))
    };
}

/// Creates a `Builtin` for a constant.
macro_rules! constant {
    ($value:expr) => {
        |stack| {
            stack.pushn($value);
            Ok(())
        }
    };
}

/// Returns a table of builtin names and the functions that implement them.
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn table() -> HashMap<&'static str, Builtin> {
    HashMap::from([
        // Arithmetic
        ("+", builtin_add as Builtin),
        ("-", builtin_sub),
        ("*", builtin_mul),
        ("/", builtin_div),
        // Trigonometric
        ("sin", builtin_sin),
        ("cos", builtin_cos),
        ("tan", builtin_tan),
        ("asin", builtin_asin),
        ("acos", builtin_acos),
        ("atan", builtin_atan),
        // Constants
        (
            "c",
            constant!(
                Number::new(299_792_458.0).with_unit((&units::METER / &units::SECOND).unwrap())
            ),
        ),
        ("e", constant!(Number::new(std::f64::consts::E))),
        (
            "h",
            constant!(
                Number::new(6.626_070_15e-34).with_unit((&*units::JOULE * &units::SECOND).unwrap())
            ),
        ),
        (
            "hbar",
            constant!(Number::new(1.054_571_817e-34)
                .with_unit((&*units::JOULE * &units::SECOND).unwrap())),
        ),
        ("pi", constant!(Number::new(std::f64::consts::PI))),
        // Stack
        ("clear", builtin_clear),
        ("dup", builtin_dup),
        ("keep", builtin_keep),
        ("pop", builtin_pop),
        ("swap", builtin_swap),
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
        unit!(&*units::NEWTON),
        unit!(&*units::JOULE),
        unit!(&*units::WATT),
    ])
}
