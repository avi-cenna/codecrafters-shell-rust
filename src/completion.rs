use crate::commands::BuiltinCmd;
use crate::filesystem::{get_all_paths_in_dir, PathInfo, is_executable};
use itertools::Itertools;
use rustyline::Helper;
use rustyline::completion::Completer;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use std::ffi::OsStr;

/// A shell completer for both commands and filepaths
pub struct HybridCompleter;

impl Completer for HybridCompleter {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<String>)> {
        let (start, word) = extract_word(line, pos);
        let is_first_word = line[..start].trim().is_empty();
        let candidates: (usize, Vec<String>) = if is_first_word {
            let cand = if let Some(path) = std::env::var_os("PATH") {
                find_command_candidates(word, &path)
            } else {
                vec![]
            };
            (start, cand)
        } else {
            // Note: right now path completions only work for unquoted paths without spaces. This
            // satisfies the CodeCrafters spec, but it might be worth adding logic to handle
            // quoting and spaces at some point.
            (start, find_path_completions(word))
        };
        Ok(candidates)
    }
}

fn find_command_candidates(word: &str, system_path: &OsStr) -> Vec<String> {
    let builtins = BuiltinCmd::all_names();
    let builtin_cand = builtins
        .iter()
        .filter(|builtin_name| builtin_name.starts_with(word))
        .map(ToString::to_string);

    let dirs = std::env::split_paths(&system_path);
    let entries: Vec<PathInfo> = dirs
        .filter_map(|p| get_all_paths_in_dir(&p).ok())
        .flatten()
        .filter(|p| is_executable(&p.path))
        .collect();
    let executable_cand = entries
        .into_iter()
        .map(|x| x.file_name)
        .filter(|p| p.starts_with(word));

    let combined: Vec<String> = builtin_cand
        .into_iter()
        .chain(executable_cand)
        .map(|s| s + " ")
        .unique()
        .sorted()
        .collect();

    combined
}

fn find_path_completions(word: &str) -> Vec<String> {
    let (stem, prefix) = match word.rfind('/') {
        Some(i) => (&word[..=i], &word[i + 1..]),
        None => ("", word),
    };
    let search_dir = if stem.is_empty() { "." } else { stem };
    let Ok(path_infos) = get_all_paths_in_dir(search_dir) else {
        return vec![];
    };

    let filename_candidates: Vec<String> = path_infos
        .into_iter()
        .filter(|s| s.file_name.starts_with(prefix))
        .map(|s| {
            if s.file_type.is_dir() {
                format!("{stem}{}/", s.file_name)
            } else {
                format!("{stem}{} ", s.file_name)
            }
        })
        .sorted()
        .collect();

    filename_candidates
}

impl Hinter for HybridCompleter {
    type Hint = String;
}
impl Helper for HybridCompleter {}
impl Highlighter for HybridCompleter {}
impl Validator for HybridCompleter {}

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
    use std::ffi::OsString;
    use super::*;

    /// Creates a fresh scratch dir holding an empty executable file per name, and
    /// returns it as a one-entry PATH.
    fn fake_path(test_name: &str, executables: &[&str]) -> OsString {
        use std::os::unix::fs::PermissionsExt;
        let dir =
            std::env::temp_dir().join(format!("shell-comp-{}-{test_name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for name in executables {
            let file = dir.join(name);
            std::fs::write(&file, "").unwrap();
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        dir.into_os_string()
    }

    #[test]
    fn test_command_candidates() {
        let empty = fake_path("empty", &[]);
        assert_eq!(find_command_candidates("pw", &empty), ["pwd "]);

        let rust = fake_path(
            "rust",
            &[
                "rust-analyzer",
                "rust-gdb",
                "rust-gdbgui",
                "rust-lldb",
                "rustc",
            ],
        );
        assert_eq!(
            find_command_candidates("rust-", &rust),
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
            ("tree", 4, (0, "tree")),
            ("tree ", 5, (5, "")),
            ("cat test\\ file", 14, (4, "test\\ file")),
        ];
        for (input, pos, expected) in cases {
            assert_eq!(extract_word(input, pos), expected, "input: {input:?}");
        }
    }

    #[test]
    fn test_path_completions() {
        assert_eq!(find_path_completions("src/m"), ["src/main.rs "]);
        assert_eq!(find_path_completions("sr"), ["src/"]);
        assert_eq!(find_path_completions("target/d"), ["target/debug/"]);
        assert_eq!(
            find_path_completions("Carg"),
            ["Cargo.lock ", "Cargo.toml "]
        );
    }
}
