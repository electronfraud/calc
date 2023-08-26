//! Input evaluation.

use std::string::ToString;

use crate::{builtins, integer, stack::Stack};

/// An evaluation context.
pub struct Context {
    pub stack: Stack,
    builtins: builtins::Table,
}

/// The result of an evaluation.
#[derive(Debug, PartialEq)]
pub enum Status {
    Ok,
    Halt,
    Err { error: Error, word: String },
}

/// An error that occurred during evaluation.
#[derive(Debug, PartialEq)]
pub enum Error {
    Builtins(builtins::Error),
    UnknownWord,
}

impl Context {
    /// Creates a new evaluation context with an empty stack and all of the
    /// builtins.
    #[must_use]
    pub fn new() -> Context {
        Context {
            stack: Stack::new(),
            builtins: builtins::table(),
        }
    }

    /// Evaluates a line of input. Returns false if `exit` or `q` are
    /// evaluated.
    pub fn eval(&mut self, input: &str) -> Status {
        for token in Token::split(input) {
            match token {
                Token::Float(n) => self.eval_float(n),
                Token::Integer(b) => self.eval_integer(b),
                Token::Word(w) => {
                    if w == "exit" || w == "q" {
                        return Status::Halt;
                    }
                    if let Err(e) = self.eval_word(w.as_str()) {
                        return Status::Err { error: e, word: w };
                    }
                }
            };
        }
        Status::Ok
    }

    /// Evaluates an integer by pushing it onto the stack.
    fn eval_integer(&mut self, x: integer::Integer) {
        self.stack.pushi(x);
    }

    /// Evaluates a floating point number token by pushing it onto the stack.
    fn eval_float(&mut self, x: f64) {
        self.stack.pushx(x);
    }

    /// Evaluates a word token by looking for a builtin with the name contained
    /// in the token and executing it.
    ///
    /// # Errors
    /// Returns an error if:
    /// - no builtin named `w` exists; or,
    /// - the builtin returns an error.
    fn eval_word(&mut self, w: &str) -> Result<(), Error> {
        if let Some(f) = self.builtins.get(w) {
            if let Err(e) = f(&mut self.stack) {
                Err(Error::Builtins(e))
            } else {
                Ok(())
            }
        } else {
            Err(Error::UnknownWord)
        }
    }

    /// Returns the names of all the builtins, in no particular order.
    pub fn builtin_names(&self) -> Vec<String> {
        self.builtins.keys().map(ToString::to_string).collect()
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// A token parsed from user input.
enum Token {
    Float(f64),
    Integer(integer::Integer),
    Word(String),
}

impl Token {
    /// Splits a string into a sequence of tokens.
    fn split(s: &str) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        for word in s.split_ascii_whitespace() {
            if let Some(x) = integer::Integer::parse(word) {
                tokens.push(Token::Integer(x));
            } else if let Ok(x) = word.replace(',', "").parse::<f64>() {
                tokens.push(Token::Float(x));
            } else {
                tokens.push(Token::Word(String::from(word)));
            }
        }
        tokens
    }
}
