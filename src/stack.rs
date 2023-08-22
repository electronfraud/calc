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

//! The stack.
//!
//!
//!
//! # Examples
//!
//! Stacks can be used in transactions so that if an operation on a popped
//! value fails, the transaction can be rolled back and the value remains on
//! the stack. Transactions are automatically rolled back if they fall out of
//! scope without being committed.
//!
//! ```
//! use calc::stack::{Error, Item::Float, Stack};
//! use calc::{commit, popff};
//!
//! fn div(stack: &mut Stack) -> Result<(), Error> {
//!     let mut tx = stack.begin();
//!
//!     let (a, b) = popff!(tx)?;
//!
//!     if b.value == 0.0 {
//!         return Err(Error::TypeMismatch);
//!     }
//!
//!     tx.pushx(a.value / b.value);
//!     commit!(tx)
//! }
//!
//! let mut stack = Stack::new();
//!
//! stack.pushx(1.0);
//! stack.pushx(0.0);
//!
//! assert!(div(&mut stack).is_err());
//! assert_eq!(stack.height(), 2);
//!
//! stack.pop();
//! stack.pushx(2.0);
//!
//! assert!(div(&mut stack).is_ok());
//! assert_eq!(stack.height(), 1);
//! ```

use crate::{integer, units};

/// Errors returned by stack operations.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Returned when there aren't enough items on the stack.
    Underflow,
    /// Returned when the items on the stack don't have the required types.
    TypeMismatch,
    /// Returned when an integer is required but a floating-point value has a
    /// fractional component.
    NotAnInteger,
    /// Returned when a dimensionless number is required but a value has units.
    NotDimensionless,
}

/// An item on the stack.
#[derive(Clone, Debug)]
pub enum Item {
    Float(units::Number),
    Integer(integer::Integer),
    Unit(units::Unit),
}

/// A LIFO collection of typed objects.
///
pub struct Stack(Vec<Item>);

impl Stack {
    /// Creates an empty stack.
    #[must_use]
    pub fn new() -> Stack {
        Stack(Vec::new())
    }

    /// Returns the number of items on the stack.
    #[must_use]
    pub fn height(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the stack is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Removes all items from the stack.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Pops an item off of the stack and returns it. Returns `None` if the
    /// stack is empty.
    pub fn pop(&mut self) -> Option<Item> {
        self.0.pop()
    }

    /// Pushes an item onto the stack.
    pub fn push(&mut self, item: Item) {
        self.0.push(item);
    }

    /// Pushes a floating-point number with optional units onto the stack.
    pub fn pushf(&mut self, x: units::Number) {
        self.0.push(Item::Float(x));
    }

    /// Pushes a unit onto the stack.
    pub fn pushu(&mut self, u: units::Unit) {
        self.0.push(Item::Unit(u));
    }

    /// Pushes a dimensionless floating-point number onto the stack.
    pub fn pushx(&mut self, x: f64) {
        self.pushf(units::Number::new(x));
    }

    /// Pushes an integer onto the stack.
    pub fn pushi(&mut self, x: integer::Integer) {
        self.push(Item::Integer(x));
    }

    /// Starts a transaction.
    pub fn begin(&mut self) -> Transaction {
        let stack_remaining = self.height();
        Transaction {
            stack: self,
            stack_remaining,
            pushed: Vec::new(),
        }
    }
}

impl Default for Stack {
    /// The default value for a `Stack` is an empty stack.
    fn default() -> Self {
        Self::new()
    }
}

/// Interface to a stack transaction.
pub struct Transaction<'a> {
    stack: &'a mut Stack,
    stack_remaining: usize,
    pushed: Vec<Item>,
}

impl Transaction<'_> {
    /// Returns the number of items on the stack.
    #[must_use]
    pub fn height(&self) -> usize {
        self.stack_remaining + self.pushed.len()
    }

    /// Returns true if the stack has no items on it.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.height() == 0
    }

    /// Pops an item off the stack.
    ///
    /// Use the `popb`, `popn`, and `popu` macros to pop an item and perform
    /// type-checking.
    ///
    /// # Errors
    ///
    /// Returns an error if the stack is empty.
    pub fn pop(&mut self) -> Result<Item, Error> {
        if self.is_empty() {
            Err(Error::Underflow)
        } else {
            Ok(self.pushed.pop().unwrap_or_else(|| {
                self.stack_remaining -= 1;
                self.stack.0[self.stack_remaining].clone()
            }))
        }
    }

    /// Pops two items off the stack and returns them.
    ///
    /// Use the `popbb`, `popnn`, and `popnu` macros to pop items and perform
    /// type-checking.
    ///
    /// # Errors
    ///
    /// Returns an error if there are fewer than two items on the stack.
    pub fn pop2(&mut self) -> Result<(Item, Item), Error> {
        if self.height() < 2 {
            return Err(Error::Underflow);
        }

        let b = self.pushed.pop().unwrap_or_else(|| {
            self.stack_remaining -= 1;
            self.stack.0[self.stack_remaining].clone()
        });
        let a = self.pushed.pop().unwrap_or_else(|| {
            self.stack_remaining -= 1;
            self.stack.0[self.stack_remaining].clone()
        });

        Ok((a, b))
    }

    /// Removes everything from the stack except the topmost `n` items.
    ///
    /// # Errors
    ///
    /// Returns an error if there are fewer than `n` items on the stack.
    pub fn keep(&mut self, n: usize) -> Result<(), Error> {
        if self.height() < n {
            return Err(Error::Underflow);
        }

        let mut new_pushed: Vec<Item> = Vec::with_capacity(n);
        if self.stack_remaining > 0 && n > self.pushed.len() {
            let n_from_stack = n - self.pushed.len();
            let ix0 = self.stack_remaining - n_from_stack;
            new_pushed.extend_from_slice(&self.stack.0[ix0..self.stack_remaining]);
        }
        new_pushed.append(&mut self.pushed);
        self.pushed = new_pushed;
        self.stack_remaining = 0;

        Ok(())
    }

    /// Pushes an item onto the stack.
    pub fn push(&mut self, item: Item) {
        self.pushed.push(item);
    }

    /// Pushes a floating-point number with optional units onto the stack.
    pub fn pushf(&mut self, x: units::Number) {
        self.pushed.push(Item::Float(x));
    }

    /// Pushes a unit onto the stack.
    pub fn pushu(&mut self, u: units::Unit) {
        self.pushed.push(Item::Unit(u));
    }

    /// Pushes a dimensionless floating-point number onto the stack.
    pub fn pushx(&mut self, x: f64) {
        self.pushed.push(Item::Float(units::Number::new(x)));
    }

    /// Pushes an integer onto the stack.
    pub fn pushi(&mut self, x: integer::Integer) {
        self.pushed.push(Item::Integer(x));
    }

    /// Commits all pops and pushes performed during this transaction to the
    /// stack and ends the transaction.
    ///
    /// Use the `commit!` macro for a convenient way to commit a transaction
    /// and produce an `Ok(())`.
    pub fn commit(&mut self) {
        self.stack.0.truncate(self.stack_remaining);
        self.stack.0.append(&mut self.pushed);
        self.stack_remaining = self.stack.height();
    }
}

#[doc(hidden)]
pub fn float_as_int(x: &units::Number) -> Result<integer::Integer, Error> {
    if x.value.fract() != 0.0 {
        Err(Error::NotAnInteger)
    } else if !x.is_dimensionless() {
        Err(Error::NotDimensionless)
    } else {
        #[allow(clippy::cast_possible_truncation)]
        Ok(integer::Integer::dec(x.value as i64))
    }
}

#[doc(hidden)]
pub fn zip<T, U>(a: Result<T, Error>, b: Result<U, Error>) -> Result<(T, U), Error> {
    match (a, b) {
        (Ok(a), Ok(b)) => Ok((a, b)),
        (Err(a), _) => Err(a),
        (_, Err(b)) => Err(b),
    }
}

/// Pops a numeric item off the stack. When successful, the result will always
/// be a `units::Number`, even if the popped item was an integer.
#[macro_export]
macro_rules! pop_as_f {
    ($stacklike: ident) => {
        $stacklike.pop().and_then(|item| match item {
            $crate::stack::Item::Float(x) => Ok(x),
            $crate::stack::Item::Integer(x) => Ok($crate::units::Number::new(x.value as f64)),
            _ => Err($crate::stack::Error::TypeMismatch),
        })
    };
}

/// Pops a numeric item and a unit off the stack. When successful, the numeric
/// item will always be a `units::Number`, even if the popped item was an
/// integer.
#[macro_export]
macro_rules! pop_as_fu {
    ($stacklike: ident) => {
        $stacklike.pop2().and_then(|items| match items {
            ($crate::stack::Item::Float(x), $crate::stack::Item::Unit(u)) => Ok((x, u)),
            ($crate::stack::Item::Integer(x), $crate::stack::Item::Unit(u)) => {
                Ok((x.as_units_number(), u))
            }
            _ => Err($crate::stack::Error::TypeMismatch),
        })
    };
}

/// Pops a numeric item off the stack. Floating-point numbers must not have a
/// fractional component. When successful, the result of this macro will always
/// be an `integer::Integer`.
#[macro_export]
macro_rules! pop_as_i {
    ($stacklike: ident) => {
        $stacklike.pop().and_then(|item| match item {
            $crate::stack::Item::Float(x) => $crate::stack::float_as_int(&x),
            $crate::stack::Item::Integer(x) => Ok(x),
            _ => Err($crate::stack::Error::TypeMismatch),
        })
    };
}

/// Pops two numeric items off the stack. Floating-point numbers must not have
/// fractional components. When successful, the result of this macro will
/// always be two `integer::Integer`s.
#[macro_export]
macro_rules! pop_as_ii {
    ($stacklike: ident) => {
        $stacklike.pop2().and_then(|items| match items {
            ($crate::stack::Item::Float(a), $crate::stack::Item::Float(b)) => $crate::stack::zip(
                $crate::stack::float_as_int(&a),
                $crate::stack::float_as_int(&b),
            ),
            ($crate::stack::Item::Float(a), $crate::stack::Item::Integer(b)) => {
                $crate::stack::float_as_int(&a).map(|a| (a, b))
            }
            ($crate::stack::Item::Integer(a), $crate::stack::Item::Float(b)) => {
                $crate::stack::float_as_int(&b).map(|b| (a, b))
            }
            ($crate::stack::Item::Integer(a), $crate::stack::Item::Integer(b)) => Ok((a, b)),
            _ => Err($crate::stack::Error::TypeMismatch),
        })
    };
}

/// Pops a numeric items off the stack and returns it without casting.
#[macro_export]
macro_rules! popn {
    ($stacklike: ident) => {
        $stacklike.pop().and_then(|item| match &item {
            $crate::stack::Item::Float(_) => Ok(item),
            $crate::stack::Item::Integer(_) => Ok(item),
            _ => Err($crate::stack::Error::TypeMismatch),
        })
    };
}

/// Pops two numeric items off the stack and returns them without casting.
#[macro_export]
macro_rules! popnn {
    ($stacklike: ident) => {
        $stacklike.pop2().and_then(|items| match &items {
            ($crate::stack::Item::Float(_), $crate::stack::Item::Float(_)) => Ok(items),
            ($crate::stack::Item::Float(_), $crate::stack::Item::Integer(_)) => Ok(items),
            ($crate::stack::Item::Integer(_), $crate::stack::Item::Float(_)) => Ok(items),
            ($crate::stack::Item::Integer(_), $crate::stack::Item::Integer(_)) => Ok(items),
            _ => Err($crate::stack::Error::TypeMismatch),
        })
    };
}

/// Pops a floating-point number off a stack.
#[macro_export]
macro_rules! popf {
    ($tx: ident) => {
        $tx.pop().and_then(|items| match items {
            $crate::stack::Item::Float(a) => Ok(a),
            _ => Err($crate::stack::Error::TypeMismatch),
        })
    };
}

/// Pops two floating-point numbers off a stack.
#[macro_export]
macro_rules! popff {
    ($tx: ident) => {
        $tx.pop2().and_then(|items| match items {
            ($crate::stack::Item::Float(a), $crate::stack::Item::Float(b)) => Ok((a, b)),
            _ => Err($crate::stack::Error::TypeMismatch),
        })
    };
}

/// Pops a floating-point number and a unit off a stack.
#[macro_export]
macro_rules! popfu {
    ($tx: ident) => {
        $tx.pop2().and_then(|items| match items {
            ($crate::stack::Item::Float(a), $crate::stack::Item::Unit(b)) => Ok((a, b)),
            _ => Err($crate::stack::Error::TypeMismatch),
        })
    };
}

/// Commits a transaction. Evaluates to `Ok(())`.
#[macro_export]
macro_rules! commit {
    ($tx: ident) => {{
        $tx.commit();
        Ok(())
    }};
}

/// Iterator over a stack's items.
pub struct Iter<'a> {
    items: &'a [Item],
    ix: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ix < self.items.len() {
            let item = &self.items[self.ix];
            self.ix += 1;
            Some(item)
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a Stack {
    type Item = &'a Item;
    type IntoIter = Iter<'a>;

    /// Returns an iterator over the stack's items, starting with the item on
    /// the bottom and working upward.
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            items: self.0.as_slice(),
            ix: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::stack::Stack;

    #[test]
    fn height() {
        let mut s = Stack::new();
        assert_eq!(s.height(), 0);
        s.pushx(2.5);
        assert_eq!(s.height(), 1);
        s.pushx(6.2);
        assert_eq!(s.height(), 2);
        s.pop();
        assert_eq!(s.height(), 1);
        s.pop();
        assert_eq!(s.height(), 0);
    }

    #[test]
    fn is_empty() {
        let mut s = Stack::new();
        assert!(s.is_empty());
        s.pushx(2.5);
        assert!(!s.is_empty());
        s.pushx(6.2);
        assert!(!s.is_empty());
        s.pop();
        assert!(!s.is_empty());
        s.pop();
        assert!(s.is_empty());
    }
}
