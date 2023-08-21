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

use std::collections::HashMap;

use rustyline as rl;
use rustyline::error::ReadlineError;

use calc::{binary, builtins::Builtin, stack, stack::Stack, units};

/// A token parsed from user input.
enum Token {
    Number(f64),
    Word(String),
    BinInt(binary::Integer),
}

/// Returns a REPL prompt containing the elements in the stack, e.g. "(1 2) ".
fn prompt(stack: &Stack) -> String {
    let mut prompt = String::from("(");

    for item in stack {
        match item {
            stack::Item::Number(n) => prompt.push_str(format!("{n}").as_str()),
            stack::Item::Unit(u) => prompt.push_str(format!("{u}").as_str()),
            stack::Item::BinInt(b) => prompt.push_str(format!("{b}").as_str()),
        };
        prompt.push(' ');
    }

    if !stack.is_empty() {
        prompt.pop();
    }
    prompt.push_str(") ");

    prompt
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

/// Evaluates a number token, i.e., pushes the number onto the stack.
fn eval_number(n: f64, stack: &mut Stack) {
    stack.pushv(n);
}

fn eval_bin_int(b: binary::Integer, stack: &mut Stack) {
    stack.pushb(b);
}

/// Evaluates a word token by looking for a builtin with the name contained in
/// the token and executing it. If the builtin returns an error, or if no
/// builtin is found, prints an error and returns false.
///
/// Returns `true` if evaluation succeeded.
#[must_use]
fn eval_word(w: &str, stack: &mut Stack, builtins: &HashMap<&'static str, Builtin>) -> bool {
    if let Some(f) = builtins.get(w) {
        if let Err(e) = f(stack) {
            print!("{w}: ");
            match e {
                calc::builtins::Error::Stack(e) => match e {
                    stack::Error::TypeMismatch => println!("type mismatch"),
                    stack::Error::Underflow => println!("stack underflow"),
                },
                calc::builtins::Error::Units(e) => match e {
                    units::Error::IncommensurableUnits(_, _) => println!("incommensurable units"),
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

/// Autocompletion helper.
struct Completer {
    builtins: Vec<String>,
}

impl rl::Helper for Completer {}
impl rl::highlight::Highlighter for Completer {}
impl rl::validate::Validator for Completer {}
impl rl::hint::Hinter for Completer {
    type Hint = String;
}

impl rl::completion::Completer for Completer {
    type Candidate = String;

    /// Autocompletes builtins.
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rl::Context<'_>,
    ) -> rl::Result<(usize, Vec<String>)> {
        // Find the index of the start of the token under the cursor.
        let start = line[0..pos].rfind(char::is_whitespace).map_or(0, |p| p + 1);

        if start == pos {
            return Ok((0, vec![]));
        }

        // Find all builtins that start with the token under the cursor.
        let mut candidates: Vec<String> = vec![];
        let prefix = &line[start..pos];

        for word in &self.builtins {
            if word.starts_with(prefix) {
                candidates.push(word.clone());
            }
        }

        Ok((start, candidates))
    }
}

fn main() -> Result<(), ReadlineError> {
    // Create the evaluation context.
    let mut stack = Stack::new();
    let builtins = calc::builtins::table();

    // Initialize readline.
    let mut rl: rl::Editor<Completer, rl::history::DefaultHistory> = rl::Editor::with_config(
        rl::config::Config::builder()
            .max_history_size(1000)?
            .auto_add_history(true)
            .edit_mode(rl::config::EditMode::Emacs)
            .completion_type(rl::config::CompletionType::List)
            .build(),
    )?;

    // Load readline history.
    let history_path = home::home_dir().map(|mut path| {
        path.push(".calc_history");
        path
    });
    history_path.as_ref().map(|path| rl.load_history(&path));

    // Set up autocomplete.
    let mut completer = Completer { builtins: vec![] };
    for k in builtins.keys() {
        completer.builtins.push((*k).to_string());
    }
    completer.builtins.sort();
    rl.set_helper(Some(completer));

    // Run the REPL.
    loop {
        // Read
        let input = match rl.readline(prompt(&stack).as_str()) {
            Ok(s) => s,
            Err(ReadlineError::Eof) => return Ok(()), // normal end of input; exit Ok
            Err(e) => return Err(e),
        };

        history_path.as_ref().map(|path| rl.append_history(&path));

        // Evaluate
        for token in tokenize(input.as_str()) {
            match token {
                Token::Number(n) => eval_number(n, &mut stack),
                Token::BinInt(b) => eval_bin_int(b, &mut stack),
                Token::Word(w) => {
                    if w == "exit" || w == "q" {
                        return Ok(());
                    }
                    if !eval_word(w.as_str(), &mut stack, &builtins) {
                        break;
                    }
                }
            };
        }
    }
}
