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

use crate::stack;
use crate::stack::Stack;
use crate::units;
use crate::units::Number;
use crate::units::Unit;
use crate::{pop, pop2};

pub enum Error {
    /// A stack error that occurred while executing a builtin.
    Stack(stack::Error),
    /// A units error that occurred while executing a builtin.
    Units(units::Error),
}

/// A function that implements a builtin.
pub type Builtin = fn(&mut Stack) -> Result<(), Error>;

/// `( a b -- a+b )` Pops two items, adds them, and pushes the result.
///
/// # Errors
///
/// An error occurs if:
/// - there are fewer than two items on the stack;
/// - the items are not numbers; or,
/// - the items have incommensurable units.
///
/// If there is an error, the stack is unchanged; the operands are not popped.
pub fn builtin_add(stack: &mut Stack) -> Result<(), Error> {
    match pop2!(stack, stack::Item::Number, stack::Item::Number) {
        Ok((a, b)) => match &a + &b {
            Ok(c) => {
                stack.push_number(c);
                Ok(())
            }
            Err(e) => {
                stack.push_number(a);
                stack.push_number(b);
                Err(Error::Units(e))
            }
        },
        Err(e) => Err(Error::Stack(e)),
    }
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
///
/// If there is an error, the stack is unchanged; the operands are not popped.
pub fn builtin_sub(stack: &mut Stack) -> Result<(), Error> {
    match pop2!(stack, stack::Item::Number, stack::Item::Number) {
        Ok((a, b)) => match &a - &b {
            Ok(c) => {
                stack.push_number(c);
                Ok(())
            }
            Err(e) => {
                stack.push_number(a);
                stack.push_number(b);
                Err(Error::Units(e))
            }
        },
        Err(e) => Err(Error::Stack(e)),
    }
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
pub fn builtin_mul(stack: &mut Stack) -> Result<(), Error> {
    // Two numbers
    match pop2!(stack, stack::Item::Number, stack::Item::Number) {
        Ok((a, b)) => {
            stack.push_number(&a * &b);
            return Ok(());
        }
        Err(stack::Error::TypeMismatch) => { /* do nothing */ }
        Err(e) => return Err(Error::Stack(e)),
    }

    // Two units
    match pop2!(stack, stack::Item::Unit, stack::Item::Unit) {
        Ok((u1, u2)) => {
            stack.push_unit(&u1 * &u2);
            return Ok(());
        }
        Err(stack::Error::TypeMismatch) => { /* do nothing */ }
        Err(e) => return Err(Error::Stack(e)),
    }

    // `a` is a number and `b` is a unit
    match pop2!(stack, stack::Item::Number, stack::Item::Unit) {
        Ok((n, u)) => {
            stack.push_number(&n * &u);
            Ok(())
        }
        Err(e) => Err(Error::Stack(e)),
    }
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
/// - the items are not two units; or,
/// - the items are not a number `a` and a unit `b`.
///
/// If there is an error, the stack is unchanged; the operands are not popped.
pub fn builtin_div(stack: &mut Stack) -> Result<(), Error> {
    // Two numbers
    match pop2!(stack, stack::Item::Number, stack::Item::Number) {
        Ok((a, b)) => {
            stack.push_number(&a / &b);
            return Ok(());
        }
        Err(stack::Error::TypeMismatch) => { /* do nothing */ }
        Err(e) => return Err(Error::Stack(e)),
    }

    // Two units
    match pop2!(stack, stack::Item::Unit, stack::Item::Unit) {
        Ok((u1, u2)) => {
            stack.push_unit(&u1 / &u2);
            return Ok(());
        }
        Err(stack::Error::TypeMismatch) => { /* do nothing */ }
        Err(e) => return Err(Error::Stack(e)),
    }

    // `a` is a number and `b` is a unit
    match pop2!(stack, stack::Item::Number, stack::Item::Unit) {
        Ok((n, u)) => {
            stack.push_number(&n / &u);
            Ok(())
        }
        Err(e) => Err(Error::Stack(e)),
    }
}

/// `( ... -- )` Pops everything from the stack.
///
/// # Errors
///
/// Never returns an error.
pub fn builtin_clear(stack: &mut Stack) -> Result<(), Error> {
    stack.clear();
    Ok(())
}

/// `( [n u] -- n )` Makes a number dimensionless.
///
/// # Errors
///
/// An error occurs if:
/// - the stack is empty; or,
/// - the item on top of the stack is not a number.
pub fn builtin_drop(stack: &mut Stack) -> Result<(), Error> {
    pop!(stack, stack::Item::Number)
        .map_err(Error::Stack)
        .map(|n| stack.push_value(n.value))
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
pub fn builtin_into(stack: &mut Stack) -> Result<(), Error> {
    match pop2!(stack, stack::Item::Number, stack::Item::Unit) {
        Ok((n, u)) => {
            if let Some(n_unit) = n.unit.as_ref() {
                match n_unit.convert_interval(n.value, &u) {
                    Ok(value) => {
                        stack.push_number(Number::new(value).with_unit(u));
                        Ok(())
                    }
                    Err(e) => {
                        stack.push_number(n);
                        stack.push_unit(u);
                        Err(Error::Units(e))
                    }
                }
            } else {
                stack.push_number(n.with_unit(u));
                Ok(())
            }
        }
        Err(e) => Err(Error::Stack(e)),
    }
}

/// `( a -- )` Pops an item off the stack.
///
/// # Errors
///
/// Returns an error if the stack is empty.
pub fn builtin_pop(stack: &mut Stack) -> Result<(), Error> {
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
pub fn builtin_swap(stack: &mut Stack) -> Result<(), Error> {
    if stack.height() < 2 {
        Err(Error::Stack(stack::Error::Underflow))
    } else {
        let b = stack.pop().unwrap();
        let a = stack.pop().unwrap();
        stack.push(b);
        stack.push(a);
        Ok(())
    }
}

/// Builtin for words that are units.
///
/// If the item on top of the stack is a dimensionless number, that number is
/// assigned the unit `u`. Otherwise, `u` is pushed onto the stack.
pub fn builtin_unit(u: &Unit, stack: &mut Stack) {
    if let Some(stack::Item::Number(n)) = stack.top() {
        if n.is_dimensionless() {
            let n = n.with_unit(u.clone());
            stack.pop();
            stack.push_number(n);
            return;
        }
    }
    stack.push_unit(u.clone());
}

/// Creates a `Builtin` for a `Base`.
macro_rules! base {
    ($b:expr) => {
        ($b.symbol, anonunit!(&Unit::new(&[&$b], &[])))
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
            stack.push_number($value);
            Ok(())
        }
    };
}

/// Returns a table of builtin names and the functions that implement them.
#[must_use]
pub fn table() -> HashMap<&'static str, Builtin> {
    HashMap::from([
        // Arithmetic
        ("+", builtin_add as Builtin),
        ("-", builtin_sub),
        ("*", builtin_mul),
        ("/", builtin_div),
        // Constants
        (
            "c",
            constant!(Number::new(299_792_458.0).with_unit(&units::METER / &units::SECOND)),
        ),
        ("e", constant!(Number::new(std::f64::consts::E))),
        (
            "h",
            constant!(Number::new(6.626_070_15e-34).with_unit(&*units::JOULE * &units::SECOND)),
        ),
        (
            "hbar",
            constant!(Number::new(1.054_571_817e-34).with_unit(&*units::JOULE * &units::SECOND)),
        ),
        ("pi", constant!(Number::new(std::f64::consts::PI))),
        // Stack
        ("clear", builtin_clear),
        ("pop", builtin_pop),
        ("swap", builtin_swap),
        // Unit Conversion
        ("drop", builtin_drop),
        ("into", builtin_into),
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
        base!(units::CELSIUS),
        base!(units::FAHRENHEIT),
        base!(units::RANKINE),
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
