mod builtins;
mod parser;

use crate::builtins::{run_cd, run_echo, run_program, run_pwd, run_type};
use crate::parser::{BuiltinCmd, ShellCmd};
use anyhow::Result;
use std::io::{self, Write};

fn main() -> Result<()> {
    repl_loop()?;
    Ok(())
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
        let command_line = command_line.trim();
        let mut iter = command_line.split_whitespace();
        let cmd_name = iter.next().unwrap_or("");
        let args: Vec<String> = iter.map(ToString::to_string).collect();
        let shell_cmd = parser::parse_cmd_name(cmd_name);
        let cmd_result: Result<()> = match shell_cmd {
            ShellCmd::Builtin(builtin) => match builtin {
                BuiltinCmd::Cd => run_cd(&args),
                BuiltinCmd::Echo => {
                    run_echo(&args);
                    Ok(())
                },
                BuiltinCmd::Exit => break,
                BuiltinCmd::Pwd => run_pwd(&args),
                BuiltinCmd::Type => {
                    run_type(args);
                    Ok(())
                },
            },
            ShellCmd::Program(p) => run_program(p, args),
        };
        if let Some(err) = cmd_result.err() {
            println!("{cmd_name}: {err}");
        }
    }
    Ok(())
}
