[![progress-banner](https://backend.codecrafters.io/progress/shell/98f96b55-76cb-4c84-8f58-8b89f1e5c15f)](https://app.codecrafters.io/users/avi-cenna?r=2qF)


This repo is a small POSIX-style shell in Rust, built as a learning project through CodeCrafters.

## Features

- Shell builtins such as `echo`, `type`, `pwd`, and `cd`
- Running external programs via `PATH` lookup
- Single and double quoting with backslash escapes
- Output redirection using operators like `>`, `>>`, `2>`, `2>>`
- Tab completion for commands and file paths

## How it’s built

- Commands are tokenized via a hand-written lexer with one method per grammar rule, then parsed into the `ShellCmd` struct, and finally dispatched to a builtin or an external program.
- Redirect targets are opened once. Builtins write through `Write`, and external programs inherit the file descriptor via `Stdio`.
- Testing: unit tests for the lexer, parser, and completion, plus a black-box harness that drives the binary and checks output and redirect files.
- Known limits: operators need surrounding spaces (`echo hi > f`, not `echo hi>f`), and completion doesn’t handle quoted or space-containing filenames.
- Run with `cargo run [--release]`; test with `cargo test`.

## What I learned

The most interesting thing I learned was how to write a lexer and parser by hand. I also gained greater familiarity with file descriptors and redirection, PATH resolution, shell quoting rules, and Rust ownership rules.

## AI usage

I designed and wrote the shell logic and test cases almost entirely by hand. I used Claude for code review, explaining Rust compiler errors, test harnesses, and the exploratory tests in `tests/learning.rs`.
