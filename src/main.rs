// #[allow(unused_imports)]
use anyhow::{Result, bail};
use std::io::{self, Write};

fn main() -> Result<()> {
    repl_loop()?;
    io::stdout().flush()?;
    Ok(())
}

const BUILTINS: [&str; 4] = ["echo", "pwd", "exit", "type"];

fn repl_loop() -> Result<()> {
    loop {
        print!("$ ");
        io::stdout().flush()?;

        let mut command_line = String::new();
        let bytes_read = io::stdin().read_line(&mut command_line)?;
        if bytes_read <= 0 {
            break;
        }
        let command_line = command_line.trim();
        // let cmd_tokens = parse_cmd(command_line);
        // let cmd = cmd_tokens.get(0);
        let shell_cmd = parse_cmd(command_line)?;
        match shell_cmd {
            ShellCmd::Empty => continue,
            ShellCmd::Exit => break,
            ShellCmd::Echo(args) => {
                println!("{}", args.join(" "))
            }
            ShellCmd::Type(fname) => {
                if BUILTINS.contains(&fname.as_str()) {
                    println!("{} is a shell builtin", fname)
                } else {
                    println!("{}: not found", fname)
                }
            }
            ShellCmd::Unknown => {
                println!("{}: command not found", command_line)
            }
        }
    }
    Ok(())
}
// Some(cmd) => match cmd.as_str() {
//     "exit" => break,
//     "type" => {
//     }
// },
// None => continue,

enum ShellCmd {
    Empty,
    Exit,
    Echo(Vec<String>),
    Type(String),
    Unknown,
}

// old
// fn parse_cmd(cmd: &str) -> Vec<String> {
//     let tokens: Vec<&str> = cmd.split_whitespace().collect();
//     tokens.iter().map(|x| x.to_string()).collect()
// }

// TODO: n2 What do you think about this function?
fn parse_cmd(cmd: &str) -> Result<ShellCmd> {
    let mut iter = cmd.split_whitespace();
    let first = iter.next();
    let rest: Vec<&str> = iter.collect();
    let shell_cmd = match first {
        Some("exit") => ShellCmd::Exit,
        Some("echo") => {
            let rest: Vec<String> = rest.into_iter().map(|x| x.to_string()).collect();
            ShellCmd::Echo(rest)
        }
        Some("type") => {
            let rest: Vec<String> = rest.into_iter().map(|x| x.to_string()).collect();
            if rest.len() != 1 {
                bail!("parse error: invalid number of arguments to type")
            }
            ShellCmd::Type(rest[0].clone())
        }
        Some("") => ShellCmd::Empty,
        Some(_) => ShellCmd::Unknown,
        None => ShellCmd::Empty,
    };
    Ok(shell_cmd)
}
