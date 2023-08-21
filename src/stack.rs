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
/// stack.push_value(1.0);
/// stack.push_value(0.0);
///
/// assert!(div(&mut stack).is_err());
/// assert_eq!(stack.height(), 2);
///
/// stack.pop();
/// stack.push_value(2.0);
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

    /// Returns a reference to the item at the top of the stack, or `None` if
    /// the stack is empty.
    #[must_use]
    pub fn top(&self) -> Option<&Item> {
        self.0.last()
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
    pub fn push_number(&mut self, n: units::Number) {
        self.0.push(Item::Number(n));
    }

    /// Pushes a unit onto the stack.
    pub fn push_unit(&mut self, u: units::Unit) {
        self.0.push(Item::Unit(u));
    }

    /// Pushes a dimensionless number onto the stack.
    pub fn push_value(&mut self, v: f64) {
        self.push_number(units::Number::new(v));
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

pub struct Transaction<'a> {
    stack: &'a mut Stack,
    stack_remaining: usize,
    pushed: Vec<Item>,
}

impl Transaction<'_> {
    /// Pops an item off the stack. Returns None if the stack is empty.
    pub fn pop(&mut self) -> Option<Item> {
        if self.is_empty() {
            return None;
        }

        self.pushed.pop().or_else(|| {
            self.stack_remaining -= 1;
            Some(self.stack.0[self.stack_remaining].clone())
        })
    }

    /// Pops two items off the stack. Returns None if there are fewer than two
    /// items on the stack.
    pub fn pop2(&mut self) -> Option<(Item, Item)> {
        if self.height() < 2 {
            return None;
        }

        let b = self.pushed.pop().or_else(|| {
            self.stack_remaining -= 1;
            Some(self.stack.0[self.stack_remaining].clone())
        });
        let a = self.pushed.pop().or_else(|| {
            self.stack_remaining -= 1;
            Some(self.stack.0[self.stack_remaining].clone())
        });

        a.zip(b)
    }

    /// Pushes a number with units onto the stack.
    pub fn pushn(&mut self, n: units::Number) {
        self.pushed.push(Item::Number(n));
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
}

/// Commit a transaction and produce an Ok(()) value.
#[macro_export]
macro_rules! commit {
    ($tx: ident) => {{
        $tx.commit();
        Ok(())
    }};
}

/// Pop two numbers off a stack in a transaction.
#[macro_export]
macro_rules! popnn {
    ($tx: ident) => {
        $tx.pop2()
            .map(|items| match items {
                ($crate::stack::Item::Number(a), $crate::stack::Item::Number(b)) => Ok((a, b)),
                _ => Err($crate::stack::Error::TypeMismatch),
            })
            .unwrap_or(Err($crate::stack::Error::Underflow))
    };
}

/// Pop a number off a stack in a transaction.
#[macro_export]
macro_rules! popn {
    ($tx: ident) => {
        $tx.pop()
            .map(|items| match items {
                $crate::stack::Item::Number(a) => Ok(a),
                _ => Err($crate::stack::Error::TypeMismatch),
            })
            .unwrap_or(Err($crate::stack::Error::Underflow))
    };
}

/// Pop an item with a specified type off a stack.
#[macro_export]
macro_rules! pop {
    ($stack: ident, $type: path) => {
        $stack.pop().map_or_else(
            || Err($crate::stack::Error::Underflow),
            |item| match item {
                $type(value) => Ok(value),
                _ => {
                    $stack.push(item);
                    Err($crate::stack::Error::TypeMismatch)
                }
            },
        )
    };
}

/// Pop two items with specified types off a stack.
#[macro_export]
macro_rules! pop2 {
    ($stack: ident, $a_type: path, $b_type: path) => {{
        let b = $stack.pop();
        let a = $stack.pop();
        match (a, b) {
            (Some($a_type(a)), Some($b_type(b))) => Ok((a, b)),
            (Some(a), Some(b)) => {
                $stack.push(a);
                $stack.push(b);
                Err($crate::stack::Error::TypeMismatch)
            }
            (None, Some(b)) => {
                $stack.push(b);
                Err($crate::stack::Error::Underflow)
            }
            (None, None) => Err($crate::stack::Error::Underflow),
            _ => panic!("Impossible stack situation"),
        }
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
    use crate::stack::{Error, Item, Stack};
    use crate::units;

    #[test]
    fn top() {
        let mut s = Stack::new();
        assert!(s.top().is_none());

        s.push_value(1.0);
        match s.top().unwrap() {
            Item::Number(n) => assert_eq!(n.value, 1.0),
            _ => assert!(false),
        }

        s.push_value(2.0);
        match s.top().unwrap() {
            Item::Number(n) => assert_eq!(n.value, 2.0),
            _ => assert!(false),
        }

        s.pop();
        match s.top().unwrap() {
            Item::Number(n) => assert_eq!(n.value, 1.0),
            _ => assert!(false),
        }

        s.pop();
        assert!(s.top().is_none());
    }

    #[test]
    fn height() {
        let mut s = Stack::new();
        assert_eq!(s.height(), 0);
        s.push_value(2.5);
        assert_eq!(s.height(), 1);
        s.push_value(6.2);
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
        s.push_value(2.5);
        assert!(!s.is_empty());
        s.push_value(6.2);
        assert!(!s.is_empty());
        s.pop();
        assert!(!s.is_empty());
        s.pop();
        assert!(s.is_empty());
    }

    #[test]
    fn pop_underflow() {
        let mut s = Stack::new();
        assert_eq!(pop!(s, Item::Number).unwrap_err(), Error::Underflow);
    }

    #[test]
    fn pop_type_mismatch() {
        let mut s = Stack::new();
        s.push_unit((&units::METER / &units::SECOND).unwrap());
        let h = s.height();
        assert_eq!(pop!(s, Item::Number).unwrap_err(), Error::TypeMismatch);
        assert_eq!(s.height(), h);
    }

    #[test]
    fn pop_happy() {
        let mut s = Stack::new();
        s.push_value(2.2);
        let h = s.height();
        assert_eq!(pop!(s, Item::Number).unwrap().value, 2.2);
        assert_eq!(s.height(), h - 1);
    }

    #[test]
    fn pop2_underflow() {
        let mut s = Stack::new();
        assert_eq!(
            pop2!(s, Item::Number, Item::Number).unwrap_err(),
            Error::Underflow
        );

        s.push_unit((&units::METER / &units::SECOND).unwrap());
        let h = s.height();
        assert_eq!(
            pop2!(s, Item::Number, Item::Number).unwrap_err(),
            Error::Underflow
        );
        assert_eq!(s.height(), h);
    }

    #[test]
    fn pop2_type_mismatch() {
        let mut s = Stack::new();

        s.push_unit((&units::METER / &units::SECOND).unwrap());
        s.push_unit((&units::METER / &units::SECOND).unwrap());
        let h = s.height();
        assert_eq!(
            pop2!(s, Item::Number, Item::Number).unwrap_err(),
            Error::TypeMismatch
        );
        assert_eq!(s.height(), h);

        s.push_value(2.2);
        let h = s.height();
        assert_eq!(
            pop2!(s, Item::Number, Item::Number).unwrap_err(),
            Error::TypeMismatch
        );
        assert_eq!(s.height(), h);

        s.push_unit((&units::METER / &units::SECOND).unwrap());
        let h = s.height();
        assert_eq!(
            pop2!(s, Item::Number, Item::Number).unwrap_err(),
            Error::TypeMismatch
        );
        assert_eq!(s.height(), h);
    }

    #[test]
    fn pop2_happy() {
        let mut s = Stack::new();
        s.push_value(2.2);
        s.push_value(4.4);
        let h = s.height();
        let (a, b) = pop2!(s, Item::Number, Item::Number).unwrap();
        assert_eq!(a.value, 2.2);
        assert_eq!(b.value, 4.4);
        assert_eq!(s.height(), h - 2);
    }
}
