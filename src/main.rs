mod builtins;
mod lexer;

use crate::builtins::{BuiltinCmd, run_cd, run_echo, run_program, run_pwd, run_type};
use crate::lexer::split_tokens;
use anyhow::Result;
use std::io::{self, Write};

fn main() -> Result<()> {
    repl_loop()
}

fn repl_loop() -> Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let mut command_line = String::new();
        let bytes_read = io::stdin().read_line(&mut command_line)?;
        if bytes_read == 0 {
            break;
        }
        let tokens = match split_tokens(command_line.as_str()) {
            Ok(t) => t,
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
        };
        let Some((cmd_name, args)) = tokens.split_first() else {
            continue;
        };
        let cmd_result: Result<()> = match BuiltinCmd::from_name(cmd_name) {
            Some(BuiltinCmd::Cd) => run_cd(args),
            Some(BuiltinCmd::Echo) => {
                run_echo(args);
                Ok(())
            }
            Some(BuiltinCmd::Exit) => break,
            Some(BuiltinCmd::Pwd) => run_pwd(args),
            Some(BuiltinCmd::Type) => {
                run_type(args);
                Ok(())
            }
            None => run_program(cmd_name, args),
        };
        if let Err(err) = cmd_result {
            eprintln!("{cmd_name}: {err}");
        }
    }
    Ok(())
}
