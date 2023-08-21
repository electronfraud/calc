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

use crate::units;

/// Errors returned by stack operations.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Returned when there aren't enough items on the stack.
    Underflow,
    /// Returned when the items on the stack don't have the required types.
    TypeMismatch,
}

/// A LIFO collection of numbers and units.
///
/// # Examples
///
/// Stacks can be used in transactions so that if an operation on a popped
/// value fails, the transaction can be rolled back and the value remains on
/// the stack. Transactions are automatically rolled back if they fall out of
/// scope without being committed.
///
/// ```
/// use calc::stack::{Error, Item::Number, Stack};
/// use calc::{commit, popnn};
///
/// fn div(stack: &mut Stack) -> Result<(), Error> {
///     let mut tx = stack.begin();
///
///     let (a, b) = popnn!(tx)?;
///
///     if b.value == 0.0 {
///         return Err(Error::TypeMismatch);
///     }
///
///     tx.pushv(a.value / b.value);
///     commit!(tx)
/// }
///
/// let mut stack = Stack::new();
///
/// stack.pushv(1.0);
/// stack.pushv(0.0);
///
/// assert!(div(&mut stack).is_err());
/// assert_eq!(stack.height(), 2);
///
/// stack.pop();
/// stack.pushv(2.0);
///
/// assert!(div(&mut stack).is_ok());
/// assert_eq!(stack.height(), 1);
/// ```
pub struct Stack(Vec<Item>);

/// An item on the stack.
#[derive(Clone, Debug)]
pub enum Item {
    Number(units::Number),
    Unit(units::Unit),
}

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

    /// Pushes a number onto the stack.
    pub fn pushn(&mut self, n: units::Number) {
        self.0.push(Item::Number(n));
    }

    /// Pushes a unit onto the stack.
    pub fn pushu(&mut self, u: units::Unit) {
        self.0.push(Item::Unit(u));
    }

    /// Pushes a dimensionless number onto the stack.
    pub fn pushv(&mut self, v: f64) {
        self.pushn(units::Number::new(v));
    }

    /// Start a transaction.
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
    /// # Errors
    ///
    /// Returns an error if there are fewer than two items on the stack.
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

    /// Pushes an item onto the stack.
    pub fn push(&mut self, item: Item) {
        self.pushed.push(item);
    }

    /// Pushes a number with units onto the stack.
    pub fn pushn(&mut self, n: units::Number) {
        self.pushed.push(Item::Number(n));
    }

    /// Pushes a unit onto the stack.
    pub fn pushu(&mut self, u: units::Unit) {
        self.pushed.push(Item::Unit(u));
    }

    /// Pushes a dimensionless number onto the stack.
    pub fn pushv(&mut self, n: f64) {
        self.pushed.push(Item::Number(units::Number::new(n)));
    }

    /// Commits all pops and pushes performed during this transaction to the
    /// stack and ends the transaction.
    pub fn commit(&mut self) {
        self.stack.0.truncate(self.stack_remaining);
        self.stack.0.append(&mut self.pushed);
        self.stack_remaining = self.stack.height();
    }
}

/// Pops a number off a stack.
#[macro_export]
macro_rules! popn {
    ($tx: ident) => {
        $tx.pop().and_then(|items| match items {
            $crate::stack::Item::Number(a) => Ok(a),
            _ => Err($crate::stack::Error::TypeMismatch),
        })
    };
}

/// Pops two numbers off a stack.
#[macro_export]
macro_rules! popnn {
    ($tx: ident) => {
        $tx.pop2().and_then(|items| match items {
            ($crate::stack::Item::Number(a), $crate::stack::Item::Number(b)) => Ok((a, b)),
            _ => Err($crate::stack::Error::TypeMismatch),
        })
    };
}

/// Pops a number and a unit off a stack.
#[macro_export]
macro_rules! popnu {
    ($tx: ident) => {
        $tx.pop2().and_then(|items| match items {
            ($crate::stack::Item::Number(a), $crate::stack::Item::Unit(b)) => Ok((a, b)),
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
        s.pushv(2.5);
        assert_eq!(s.height(), 1);
        s.pushv(6.2);
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
        s.pushv(2.5);
        assert!(!s.is_empty());
        s.pushv(6.2);
        assert!(!s.is_empty());
        s.pop();
        assert!(!s.is_empty());
        s.pop();
        assert!(s.is_empty());
    }
}
