use std::collections::HashMap;
use std::string::ToString;

use crate::{binary, builtins, builtins::Builtin, stack, stack::Stack, units};

/// A table of builtin function names and their implementations.
type BuiltinsTable = HashMap<&'static str, Builtin>;

/// An evaluation context.
pub struct Context {
    stack: Stack,
    builtins: BuiltinsTable,
}

/// The result of an evaluation.
#[derive(Debug, PartialEq)]
pub enum Result {
    Ok,
    Exit,
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
    pub fn eval(&mut self, input: &str) -> Result {
        for token in tokenize(input) {
            match token {
                Token::Number(n) => self.eval_number(n),
                Token::BinInt(b) => self.eval_bin_int(b),
                Token::Word(w) => {
                    if w == "exit" || w == "q" {
                        return Result::Exit;
                    }
                    if !self.eval_word(w.as_str()) {
                        break;
                    }
                }
            };
        }
        Result::Ok
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
    #[must_use]
    fn eval_word(&mut self, w: &str) -> bool {
        if let Some(f) = self.builtins.get(w) {
            if let Err(e) = f(&mut self.stack) {
                print!("{w}: ");
                match e {
                    builtins::Error::Stack(e) => match e {
                        stack::Error::TypeMismatch => println!("type mismatch"),
                        stack::Error::Underflow => println!("stack underflow"),
                    },
                    builtins::Error::Units(e) => match e {
                        units::Error::IncommensurableUnits(_, _) => {
                            println!("incommensurable units")
                        }
                        units::Error::UninvertableUnits(u) => println!("{u} can't be inverted"),
                        units::Error::NonzeroZeroPoint(b) => {
                            println!("operation would place {b} in a nonsensical position");
                        }
                    },
                };
                return false;
            }
            return true;
        }

        println!("{w}: unknown word");
        false
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

/// Converts a line of user input into a sequence of tokens.
fn tokenize(s: &str) -> Vec<Token> {
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
