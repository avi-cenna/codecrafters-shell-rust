use anyhow::Result;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::prelude::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

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
        if bytes_read == 0 {
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
            ShellCmd::Type(fnames) => run_type(fnames),
            ShellCmd::Program(p) => {
                let exit_status = run_program(p);
                exit_status.map(|_| ())
            }
            ShellCmd::Unknown(c) => {
                println!("{}: command not found", c);
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

struct ProgramArgs {
    fname: String,
    fpath: PathBuf,
    args: Vec<String>,
}

enum ShellCmd {
    Exit,
    Echo(Vec<String>),
    Type(Vec<String>),
    Empty,
    Program(ProgramArgs),
    Unknown(String),
}

fn parse_cmd(cmd: &str) -> Result<ShellCmd> {
    let mut iter = cmd.split_whitespace();
    let first = iter.next();
    let rest: Vec<String> = iter.map(|x| x.to_string()).collect();
    let shell_cmd = match first {
        Some("exit") => ShellCmd::Exit,
        Some("echo") => ShellCmd::Echo(rest),
        Some("type") => ShellCmd::Type(rest),
        // TODO: n2 (me) i want to put find_exec out of the parse.
        // all unknown should prb just be a program, and if it's isn't found, it will print the proper msg.
        Some(fname) => {
            if let Some(p) = find_executable(fname) {
                ShellCmd::Program(ProgramArgs {
                    fname: fname.to_string(),
                    fpath: p,
                    args: rest,
                })
            } else {
                ShellCmd::Unknown(fname.to_string())
            }
        }
        None => ShellCmd::Empty,
    };
    Ok(shell_cmd)
}

fn run_type(fnames: Vec<String>) -> Result<()> {
    for fname in fnames {
        if BUILTINS.contains(&fname.as_str()) {
            println!("{} is a shell builtin", fname);
            continue
        }
        if let Some(p) = find_executable(&fname) {
            println!("{} is {}", fname, p.to_str().unwrap_or(""));
        } else {
            println!("{fname}: not found");
        }
    }
    Ok(())
}

fn run_program(p: ProgramArgs) -> Result<ExitStatus> {
    let exit_status = Command::new(p.fpath).arg0(p.fname).args(p.args).status()?;
    Ok(exit_status)
}

fn find_executable(fname: &str) -> Option<PathBuf> {
    /// TODO: n2 What do you think about this function? is it idiomatic. i don't think it
    /// needed to return the Result because it's overkill to propagate the metadata error i think...
    fn is_executable(path: &Path) -> bool {
        let meta = match path.metadata() {
            Ok(meta) => meta,
            Err(e) => {
                eprintln!("Error accessing metadata for {:?}: {}", path, e);
                return false;
            }
        };
        let mode = meta.permissions().mode();
        mode & 0o111 != 0
    }

    let path = std::env::var_os("PATH").expect("PATH not set");
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(fname);
        if candidate.is_file() && is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}
