mod builtins;
mod executables;
mod lexer;
mod parser;

use crate::builtins::{BuiltinCmd, run_cd, run_echo, run_pwd, run_type};
use crate::executables::{all_executables_on_path, run_program};
use crate::lexer::split_tokens;
use crate::parser::OutputType;
use anyhow::{Context, Result};
use itertools::Itertools;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config, Editor, Helper};
use std::fs::{DirEntry, File};
use std::io::{self, Write};
use std::ops::ControlFlow;
use std::path::Path;
use std::process::Stdio;
use std::{fs, result};

fn main() -> Result<()> {
    repl_loop()
}

fn repl_loop() -> Result<()> {
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(SimpleCompleter {}));
    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                match process_line(line.as_str()) {
                    Ok(ControlFlow::Continue(())) => {}
                    Ok(ControlFlow::Break(())) => break,
                    Err(e) => eprintln!("{e:#}"),
                }
                rl.add_history_entry(line.as_str())?;
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("Error: {err:?}");
                break;
            }
        }
    }
    Ok(())
}

fn process_line(line: &str) -> Result<ControlFlow<()>> {
    let tokens = split_tokens(line)?;
    if tokens.is_empty() {
        return Ok(ControlFlow::Continue(()));
    }
    let cmd = parser::parse_tokens(tokens)?;

    let stdout_file = open_redirect(&cmd.stdout_dest)?;
    let stderr_file = open_redirect(&cmd.stderr_dest)?;

    if let Some(builtin) = BuiltinCmd::from_name(cmd.name.as_str()) {
        let mut out: Box<dyn Write> = match stdout_file {
            Some(f) => Box::new(f),
            None => Box::new(io::stdout()),
        };
        let mut err: Box<dyn Write> = match stderr_file {
            Some(f) => Box::new(f),
            None => Box::new(io::stderr()),
        };
        let result = match builtin {
            BuiltinCmd::Cd => run_cd(&cmd),
            BuiltinCmd::Echo => run_echo(&cmd, out.as_mut()),
            BuiltinCmd::Exit => return Ok(ControlFlow::Break(())),
            BuiltinCmd::Pwd => run_pwd(&cmd, out.as_mut()),
            BuiltinCmd::Type => run_type(&cmd, out.as_mut(), err.as_mut()),
        };
        if let Err(e) = result {
            writeln!(err, "{}: {e}", cmd.name.as_str())?;
        }
    } else {
        let out = stdout_file.map_or(Stdio::inherit(), Stdio::from);
        let err = stderr_file.map_or(Stdio::inherit(), Stdio::from);
        if let Err(e) = run_program(&cmd, out, err) {
            eprintln!("{}: {e}", cmd.name.as_str());
        }
    }

    Ok(ControlFlow::Continue(()))
}

fn open_redirect(output_type: &OutputType) -> Result<Option<File>> {
    Ok(match output_type {
        OutputType::Inherit => None,
        OutputType::OverwriteFile(path) => Some(File::create(path).with_context(|| path.clone())?),
        OutputType::AppendFile(path) => Some(
            File::options()
                .create(true)
                .append(true)
                .open(path)
                .with_context(|| path.clone())?,
        ),
    })
}

// completion logix. eventually move elsewhere.
pub type ReadlineResult<T> = result::Result<T, ReadlineError>;

struct SimpleCompleter;
impl Completer for SimpleCompleter {
    // note:: consider updating this to completion::Pair so we can separate
    // display logic from insertion logic (insertion requires appending a space)
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> ReadlineResult<(usize, Vec<String>)> {
        // For now, complete full line. eventually we may need to get the word under position
        Ok(completion_candidates(line, pos))
    }
}

fn completion_candidates(line: &str, pos: usize) -> (usize, Vec<String>) {
    let (s, w) = extract_word(line, pos);
    if w.trim().is_empty() {
        return (s, vec![]);
    }
    // let f = rustyline::completion::Pair{};
    // TODO: n2 is this good/idiomatic?
    // e38: yes, and it's correct now. `line[..s].trim().is_empty()` says the same
    // thing more briefly. The only difference is Unicode vs ASCII whitespace,
    // which doesn't matter here.
    let is_first_word = line[..s].chars().all(|c| c.is_ascii_whitespace());

    if is_first_word {
        let builtins = BuiltinCmd::all_names();
        let builtin_cand = builtins
            .iter()
            .filter(|builtin_name| builtin_name.starts_with(line))
            .map(ToString::to_string);
        let executables = all_executables_on_path().unwrap_or_default();
        let executable_cand = executables
            .iter()
            .map(|x| x.file_name().display().to_string())
            .filter(|p| p.starts_with(line));

        let combined: Vec<String> = builtin_cand
            .into_iter()
            .chain(executable_cand)
            // e38: possible bug once List mode is on: the TAB-TAB list prints each
            // candidate's display(), and for a String that's the same as what gets
            // inserted, trailing space included. So the list line ends in a space and
            // columns pad wider. If the tester matches that line exactly, it fails.
            // However, CodeCrafters tests pass without this e38...
            // See note above: may decide to use Pair here, but not sure yet.
            .map(|s| s + " ")
            .unique()
            .sorted()
            .collect();
        (s, combined)
    } else {
        let Ok(dir) = fs::read_dir(".") else {
            return (s, vec![]);
        };
        let entries: Vec<DirEntry> = dir
            .filter_map(Result::ok)
            .filter(|e| is_file(&e.path()))
            .collect();
        let filename_candidates: Vec<String> = entries
            .iter()
            .map(|e| e.file_name().display().to_string() + " ")
            .filter(|s| s.starts_with(w))
            .collect();
        (s, filename_candidates)
    }
}

fn is_file(f: &Path) -> bool {
    f.metadata().is_ok_and(|m| m.is_file())
}

impl Hinter for SimpleCompleter {
    type Hint = String;
}
impl Helper for SimpleCompleter {}
impl Highlighter for SimpleCompleter {}
impl Validator for SimpleCompleter {}

// filename completion; this is adapted from FilenameCompleter of rustyline

const fn is_break_char(c: char) -> bool {
    matches!(
        c,
        ' ' | '\t'
            | '\n'
            | '"'
            | '\\'
            | '\''
            | '`'
            | '@'
            | '$'
            | '>'
            | '<'
            | '='
            | ';'
            | '|'
            | '&'
            | '{'
            | '('
            | '\0'
    )
}

const ESCAPE_CHAR: char = '\\';

// In double quotes, not all break_chars need to be escaped
// https://www.gnu.org/software/bash/manual/html_node/Double-Quotes.html
const fn double_quotes_special_chars(c: char) -> bool {
    matches!(c, '"' | '$' | '\\' | '`')
}

fn extract_word(line: &str, pos: usize) -> (usize, &str) {
    let line = &line[..pos];
    if line.is_empty() {
        return (0, line);
    }

    let mut start = None;
    for (i, c) in line.char_indices().rev() {
        if start.is_some() {
            if c == ESCAPE_CHAR {
                start = None;
                continue;
            }
            break;
        }
        if is_break_char(c) {
            start = Some(i + c.len_utf8());
        }
    }

    match start {
        Some(start) => (start, &line[start..]),
        None => (0, line),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_candidates() {
        assert_eq!(completion_candidates("pw", 2).1[0..1], ["pwd "]);
        assert_eq!(
            completion_candidates("rust-", 5).1,
            ["rust-analyzer ", "rust-gdb ", "rust-gdbgui ", "rust-lldb "]
        );
    }

    #[test]
    fn test_extract_word() {
        // (input, cursor pos, expected (word start, word))
        let cases = [
            ("cat foo.tx", 10, (4, "foo.tx")),
            (r"cat fizz\ buz", 13, (4, r"fizz\ buz")),
            ("cat 'foo.tx'", 11, (5, "foo.tx")),
            ("cat \"foo.tx\"", 11, (5, "foo.tx")),
        ];
        for (input, pos, expected) in cases {
            assert_eq!(extract_word(input, pos), expected, "input: {input:?}");
        }
    }
}
