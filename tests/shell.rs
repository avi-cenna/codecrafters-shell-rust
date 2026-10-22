use std::io::{Read, Write};
use std::process::{Command, Stdio};

/// Runs the shell with `input` on stdin, closes stdin (EOF), and returns everything it
/// wrote to stdout and stderr, prompts included, exactly as the CodeCrafters tester would see it.
fn run_shell(input: &str) -> String {
    // The tester reads through a PTY, which is one stream. Hand the child the same pipe for
    // fd 1 and fd 2 (what `2>&1` does) so prompts and errors keep the order they were written in.
    let (mut reader, writer) = std::io::pipe().unwrap();
    let mut sh = Command::new(env!("CARGO_BIN_EXE_codecrafters-shell"))
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
    // (stdin, expected output). Every command line prints a "$ " prompt first, and the
    // shell prints one more prompt before it reads EOF.
    let cases: &[(&str, &str)] = &[
        ("echo hi\n", "$ hi\n$ "),
        ("\n", "$ $ "),
        ("fake-cmd-123\n", "$ fake-cmd-123: command not found\n$ "),
        ("type echo\n", "$ echo is a shell builtin\n$ "),
    ];
    for (input, expected) in cases {
        assert_eq!(run_shell(input), *expected, "input: {input:?}");
    }
}
