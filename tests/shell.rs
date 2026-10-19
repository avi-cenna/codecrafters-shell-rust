use std::io::Write;
use std::process::{Command, Stdio};

/// Runs the shell with `input` on stdin, closes stdin (EOF), and returns everything it
/// wrote to stdout, prompts included, exactly as the CodeCrafters tester would see it.
fn run_shell(input: &str) -> String {
    let mut sh = Command::new(env!("CARGO_BIN_EXE_codecrafters-shell"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    // Dropping stdin after the write sends EOF, so the REPL exits.
    sh.stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = sh.wait_with_output().unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn shell_cases() {
    // (stdin, expected stdout). Every command line prints a "$ " prompt first, and the
    // shell prints one more prompt before it reads EOF.
    let cases: &[(&str, &str)] = &[
        // ("echo hi\n", "$ hi\n$ "),
    ];
    for (input, expected) in cases {
        assert_eq!(run_shell(input), *expected, "input: {input:?}");
    }
}
