use anyhow::{Result, bail};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn main() -> Result<()> {
    repl_loop()?;
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
        let shell_cmd = parse_cmd(command_line)?;
        let cmd_result: Result<()> = match shell_cmd {
            ShellCmd::Empty => continue,
            ShellCmd::Exit => break,
            ShellCmd::Echo(args) => {
                println!("{}", args.join(" "));
                Ok(())
            }
            ShellCmd::Type(fname) => run_type(fname),
            ShellCmd::Unknown => {
                println!("{}: command not found", command_line);
                Ok(())
            }
        };
        if let Some(err) = cmd_result.err() {
            println!("Error when processing shell cmd: {err}")
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
    Type(Vec<String>),
    Unknown,
}

fn parse_cmd(cmd: &str) -> Result<ShellCmd> {
    let mut iter = cmd.split_whitespace();
    let first = iter.next();
    let rest: Vec<String> = iter.map(|x| x.to_string()).collect();
    let shell_cmd = match first {
        Some("exit") => ShellCmd::Exit,
        Some("echo") => ShellCmd::Echo(rest),
        Some("type") => ShellCmd::Type(rest),
        Some("") => ShellCmd::Empty,
        Some(_) => ShellCmd::Unknown,
        None => ShellCmd::Empty,
    };
    Ok(shell_cmd)
}

fn run_type(fnames: Vec<String>) -> Result<()> {
    for fname in fnames {
        if BUILTINS.contains(&fname.as_str()) {
            println!("{} is a shell builtin", fname);
            return Ok(());
        }
        let path = std::env::var_os("PATH").expect("PATH not set");
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join(&fname);
            if candidate.is_file() && is_executable(&candidate)? {
                println!("{} is {}", fname, candidate.to_str().unwrap_or(""));
                return Ok(());
            }
        }
        println!("{fname}: not found");
    }
    Ok(())
}

fn is_executable(path: &Path) -> Result<bool> {
    let meta = path.metadata()?;
    let mode = meta.permissions().mode();
    Ok(mode & 0o111 != 0)
}
