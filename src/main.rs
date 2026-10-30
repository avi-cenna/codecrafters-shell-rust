mod builtins;
mod executables;
mod lexer;
mod parser;
mod comp;

use crate::builtins::{BuiltinCmd, run_cd, run_echo, run_pwd, run_type};
use crate::comp::SimpleCompleter;
use crate::executables::run_program;
use crate::lexer::split_tokens;
use crate::parser::OutputType;
use anyhow::{Context, Result};
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, Editor};
use std::fs::File;
use std::io::{self, Write};
use std::ops::ControlFlow;
use std::process::Stdio;

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

