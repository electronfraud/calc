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

use rustyline as rl;
use rustyline::error::ReadlineError;

use calc::{builtins, eval, stack, units};

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

/// Prints an error message when an evaluation error occurs.
fn print_error(error: &eval::Error, word: &String) {
    print!("{word}: ");
    match error {
        eval::Error::Builtins(e) => match e {
            builtins::Error::Stack(e) => match e {
                stack::Error::TypeMismatch => println!("type mismatch"),
                stack::Error::Underflow => println!("stack underflow"),
            },
            builtins::Error::Units(e) => match e {
                units::Error::IncommensurableUnits(_, _) => {
                    println!("incommensurable units");
                }
                units::Error::UninvertableUnits(u) => println!("{u} can't be inverted"),
                units::Error::NonzeroZeroPoint(b) => {
                    println!("operation would place {b} in a nonsensical position");
                }
            },
            builtins::Error::MissingUnit => println!("missing unit"),
            builtins::Error::NotDimensionless => println!("number must be dimensionless"),
            builtins::Error::NotNonNegative => println!("number must be non-negative"),
            builtins::Error::NotWhole => println!("number must be whole"),
        },
        eval::Error::UnknownWord => println!("unknown word"),
    }
}

fn main() -> Result<(), ReadlineError> {
    // Create the evaluation context.
    let mut ctx = eval::Context::new();

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
    let mut completer = Completer {
        builtins: ctx.builtin_names(),
    };
    completer.builtins.sort();
    rl.set_helper(Some(completer));

    // Run the REPL.
    loop {
        // Read
        let input = match rl.readline(ctx.prompt().as_str()) {
            Ok(s) => s,
            Err(ReadlineError::Eof) => return Ok(()), // normal end of input; exit Ok
            Err(e) => return Err(e),
        };

        history_path.as_ref().map(|path| rl.append_history(&path));

        // Evaluate
        match ctx.eval(input.as_str()) {
            eval::Status::Ok => { /* do nothing */ }
            eval::Status::Err { error, word } => print_error(&error, &word),
            eval::Status::Exit => return Ok(()),
        }
    }
}
