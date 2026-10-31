use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Runs the shell with `input` on stdin, closes stdin (EOF), and returns everything it
/// wrote to stdout and stderr.
fn run_shell(input: &str) -> String {
    run_shell_in(Path::new("."), input)
}

/// Same as `run_shell`, but the shell starts in `dir`, so relative paths in `input` land there.
fn run_shell_in(dir: &Path, input: &str) -> String {
    let (mut reader, writer) = std::io::pipe().unwrap();
    let mut sh = Command::new(env!("CARGO_BIN_EXE_codecrafters-shell"))
        .current_dir(dir)
        .stdin(Stdio::piped())
        .stdout(writer.try_clone().unwrap())
        .stderr(writer)
        .spawn()
        .unwrap();
    // The temporary `Command` above is dropped at the end of its statement, which closes the
    // parent's copies of the write end. `read_to_string` hits EOF only once *every* write end
    // is closed, so from here on that happens exactly when the shell exits.
    // Dropping stdin after the write sends EOF, so the REPL exits.
    sh.stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let mut out = String::new();
    reader.read_to_string(&mut out).unwrap();
    sh.wait().unwrap();
    out
}

#[test]
fn shell_cases() {
    // (stdin, expected output)
    let cases: &[(&str, &str)] = &[
        ("echo hi\n", "hi\n"),
        ("\n", ""),
        ("fake-cmd-123\n", "fake-cmd-123: command not found\n"),
        ("type echo\n", "echo is a shell builtin\n"),
    ];
    for (input, expected) in cases {
        assert_eq!(run_shell(input), *expected, "input: {input:?}");
    }
}

struct RedirectCase {
    /// Files to create in the scratch dir before the shell starts: (name, contents).
    setup: &'static [(&'static str, &'static str)],
    input: &'static str,
    /// Everything that reaches the terminal (stdout + stderr)
    terminal: &'static str,
    /// Files that must exist afterward with exactly these contents: (name, contents).
    files: &'static [(&'static str, &'static str)],
}

/// A fresh, empty directory per case so cases can't see each other's files.
fn scratch_dir(case: usize) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("shell-redirect-{}-{case}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn redirect_cases() {
    // Each case runs in its own scratch dir, which is the shell's cwd, so use relative names.
    let cases: &[RedirectCase] = &[
        RedirectCase {
            setup: &[("a.txt", "abcdefgh")],
            input: "echo foo > a.txt\n",
            terminal: "",
            files: &[("a.txt", "foo\n")],
        },
        RedirectCase {
            setup: &[("e.txt", "...")],
            input: "type fizzbuzz123 2> e.txt\n",
            terminal: "",
            files: &[("e.txt", "fizzbuzz123: not found\n")],
        },
        RedirectCase {
            setup: &[("o.txt", "___")],
            input: "echo fizzbuzz >> o.txt\n",
            terminal: "",
            files: &[("o.txt", "___fizzbuzz\n")],
        },
        RedirectCase {
            setup: &[("e.txt", "...")],
            input: "type fizzbuzz123 2>> e.txt\n",
            terminal: "",
            files: &[("e.txt", "...fizzbuzz123: not found\n")],
        },
        RedirectCase {
            setup: &[("a.txt", "..."), ("e.txt", "")],
            input: "cat a.txt missing.txt 2> e.txt\n",
            terminal: "...",
            files: &[
                ("a.txt", "..."),
                ("e.txt", "cat: missing.txt: No such file or directory\n"),
            ],
        },
        RedirectCase {
            setup: &[("a.txt", "..."), ("b.txt", "")],
            input: "cat a.txt missing.txt > b.txt\n",
            terminal: "cat: missing.txt: No such file or directory\n",
            files: &[("a.txt", "..."), ("b.txt", "...")],
        },
    ];
    for (i, case) in cases.iter().enumerate() {
        let dir = scratch_dir(i);
        for (name, contents) in case.setup {
            std::fs::write(dir.join(name), contents).unwrap();
        }
        let input = case.input;
        assert_eq!(run_shell_in(&dir, input), case.terminal, "input: {input:?}");
        for (name, expected) in case.files {
            let actual = std::fs::read_to_string(dir.join(name))
                .unwrap_or_else(|e| panic!("input: {input:?}: reading {name}: {e}"));
            assert_eq!(actual, *expected, "input: {input:?}, file: {name}");
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
