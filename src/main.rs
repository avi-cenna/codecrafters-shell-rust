mod builtins;
mod lexer;
mod parser;

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
        if tokens.is_empty() {
            continue;
        };
        let cmd = match parser::parse_tokens(tokens) {
            Ok(t) => t,
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
        };
        // let Some((cmd_name, args)) = tokens.split_first() else {
        //     continue;
        // };

        // begin temp WIP logic for redirection
        // e38 (design, optional for the tester): by the time you see tokens, quoting is gone, so
        // `echo '>' x` looks like a redirect here (bash prints `> x`), and `echo hi>f` is one
        // token. Both come from the lexer not knowing `>` is an operator. Fine to defer; just
        // know where that knowledge would have to live if you ever need it.
        // let f = args.iter().position(|x| matches!(x, Token::Operator(_)));
        // e38: `type echo >` still panics here and kills the shell (index out of bounds). Bash
        // reports a syntax error and keeps going. Which slice method gives you an Option instead?
        // Related: `f` gets checked three times (`map`, `is_some`, `unwrap`). One `match` or
        // `if let` on `f` could produce both `redirect` and the trimmed `args` in one go.
        // let redirect = f.map(|x| &args[x + 1]);
        // let mut args = args;
        // if redirect.is_some() {
        //     args = &args[0..f.unwrap()];
        // }
        // let cmd_name = if let Token::Word(s) = cmd_name {
        //     s.to_string()
        // } else {
        //     panic!("foo")
        // };
        // let args: Vec<String> = args
        //     .iter()
        //     .map(|x| match x {
        //         Token::Word(s) => s.to_string(),
        //         Token::Operator(z) => "".to_string(),
        //     })
        //     .filter(|x| !x.is_empty())
        //     .collect();
        // end

        let cmd_result: Result<()> = match BuiltinCmd::from_name(cmd.name.as_str()) {
            Some(BuiltinCmd::Cd) => run_cd(&cmd.args),
            Some(BuiltinCmd::Echo) => run_echo(&cmd.args, &cmd.output_type),
            Some(BuiltinCmd::Exit) => break,
            Some(BuiltinCmd::Pwd) => run_pwd(&cmd.args, &cmd.output_type),
            Some(BuiltinCmd::Type) => run_type(&cmd.args, &cmd.output_type),
            None => run_program(cmd.name.as_str(), &cmd.args, &cmd.output_type),
        };
        if let Err(err) = cmd_result {
            eprintln!("{}: {err}", cmd.name.as_str());
        }
    }
    Ok(())
}
