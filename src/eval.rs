//! Input evaluation.

use std::string::ToString;

use crate::{binary, builtins, stack, stack::Stack};

/// An evaluation context.
pub struct Context {
    stack: Stack,
    builtins: builtins::Table,
}

/// The result of an evaluation.
#[derive(Debug, PartialEq)]
pub enum Status {
    Ok,
    Exit,
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
                Token::BinInt(b) => self.eval_bin_int(b),
                Token::Number(n) => self.eval_number(n),
                Token::Word(w) => {
                    if w == "exit" || w == "q" {
                        return Status::Exit;
                    }
                    if let Err(e) = self.eval_word(w.as_str()) {
                        return Status::Err { error: e, word: w };
                    }
                }
            };
        }
        Status::Ok
    }

    /// Evaluates a `binary::Integer`, i.e., pushes the integer onto the stack.
    fn eval_bin_int(&mut self, b: binary::Integer) {
        self.stack.pushb(b);
    }

    /// Evaluates a number token, i.e., pushes the number onto the stack.
    fn eval_number(&mut self, n: f64) {
        self.stack.pushv(n);
    }

    /// Evaluates a word token by looking for a builtin with the name contained in
    /// the token and executing it. If the builtin returns an error, or if no
    /// builtin is found, prints an error and returns false.
    ///
    /// Returns `true` if evaluation succeeded.
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

    /// Returns a REPL prompt containing the elements in the stack, e.g. "(1 2) ".
    #[must_use]
    pub fn prompt(&self) -> String {
        let mut prompt = String::from("(");

        for item in &self.stack {
            match item {
                stack::Item::Number(n) => prompt.push_str(format!("{n}").as_str()),
                stack::Item::Unit(u) => prompt.push_str(format!("{u}").as_str()),
                stack::Item::BinInt(b) => prompt.push_str(format!("{b}").as_str()),
            };
            prompt.push(' ');
        }

        if !self.stack.is_empty() {
            prompt.pop();
        }
        prompt.push_str(") ");

        prompt
    }

    /// Returns an iterator over the names of the builtins.
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
    BinInt(binary::Integer),
    Number(f64),
    Word(String),
}

impl Token {
    /// Splits a string into a sequence of tokens.
    fn split(s: &str) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        for word in s.split_ascii_whitespace() {
            if let Some(b) = binary::Integer::parse(word) {
                tokens.push(Token::BinInt(b));
            } else if let Ok(n) = word.parse::<f64>() {
                tokens.push(Token::Number(n));
            } else {
                tokens.push(Token::Word(String::from(word)));
            }
        }
        tokens
    }
}
