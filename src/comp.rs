use crate::builtins::BuiltinCmd;
use crate::executables::{PathInfo, all_executables_on_path, all_paths_in_dir};
use itertools::Itertools;
use rustyline::Helper;
use rustyline::completion::Completer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::{fs, result};

// completion logix. eventually move elsewhere.
pub type ReadlineResult<T> = result::Result<T, ReadlineError>;

pub struct SimpleCompleter;
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
    // if w.trim().is_empty() {
    //     return (s, vec![]);
    // }
    let is_first_word = line[..s].trim().is_empty();

    if is_first_word {
        let combined = compl1(line);
        (s, combined)
    } else {
        let c2 = comp2(w);
        (s, c2)
    }
}

fn compl1(line: &str) -> Vec<String> {
    let builtins = BuiltinCmd::all_names();
    let builtin_cand = builtins
        .iter()
        .filter(|builtin_name| builtin_name.starts_with(line))
        .map(ToString::to_string);
    let executables: Vec<PathInfo> = all_executables_on_path().unwrap_or_default();
    let executable_cand = executables
        .into_iter()
        .map(|x| x.file_name)
        .filter(|p| p.starts_with(line));
    let combined: Vec<String> = builtin_cand
        .into_iter()
        .chain(executable_cand)
        .map(|s| s + " ")
        .unique()
        .sorted()
        .collect();
    combined
}

fn comp2(w: &str) -> Vec<String> {
    let (stem, prefix) = match w.rfind('/') {
        Some(i) => (&w[..=i], &w[i+1..]),
        None => ("", w),
    };
    let search_dir = if stem.is_empty() { "." } else { stem };
    let search_dir = PathBuf::from(search_dir);
    let Ok(pi) = all_paths_in_dir(&search_dir) else {
        return vec![];
    };
    // let f2 = f.
    let filename_candidates: Vec<String> = pi
        .into_iter()
        .filter(|s| s.file_name.starts_with(prefix))
        .map(|s| {
            if s.file_type.is_dir() {
                format!("{stem}{}/", s.file_name)
            } else {
                format!("{stem}{} ", s.file_name)
            }
        })
        .collect();
    filename_candidates
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
        // assert_eq!(completion_candidates("pw", 2).1[0..1], ["pwd "]);
        // assert_eq!(
        //     completion_candidates("rust-", 5).1,
        //     ["rust-analyzer ", "rust-gdb ", "rust-gdbgui ", "rust-lldb "]
        // );
        assert_eq!(completion_candidates("cat Cargo.t", 11).1, ["Cargo.toml "]);
    }

    #[test]
    fn test_extract_word() {
        // (input, cursor pos, expected (word start, word))
        let cases = [
            ("cat foo.tx", 10, (4, "foo.tx")),
            (r"cat fizz\ buz", 13, (4, r"fizz\ buz")),
            ("cat 'foo.tx'", 11, (5, "foo.tx")),
            ("cat \"foo.tx\"", 11, (5, "foo.tx")),
            ("tree", 4, (0, "tree")),
            ("tree ", 5, (5, "")),
        ];
        for (input, pos, expected) in cases {
            assert_eq!(extract_word(input, pos), expected, "input: {input:?}");
        }
    }

    #[test]
    fn test_c2() {
        let a = comp2("src/");
        assert!(!a.is_empty());
            // assert_eq!(extract_word(input, pos), expected, "input: {input:?}");
    }

}
