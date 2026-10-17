use anyhow::{Result, bail};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::prelude::CommandExt;
use std::path::PathBuf;
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
            ShellCmd::Cd(path) => run_cd(path),
            ShellCmd::Empty => continue,
            ShellCmd::Exit => break,
            ShellCmd::Echo(args) => run_echo(args),
            ShellCmd::Pwd => run_pwd(),
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
            let mut iter = command_line.split_whitespace();
            let first = iter.next().unwrap_or("");
            println!("{first}: {err}")
        }
    }
    Ok(())
}

fn run_cd(path: String) -> Result<()> {
    let candidate = PathBuf::from(path);
    if !candidate.is_dir() {
        bail!("{}: No such file or directory", candidate.display())
    }
    std::env::set_current_dir(&candidate)?;
    Ok(())
}

fn run_echo(args: Vec<String>) -> Result<()> {
    println!("{}", args.join(" "));
    Ok(())
}

fn run_pwd() -> Result<()> {
    let cwd = std::env::current_dir()?;
    println!("{}", cwd.display());
    Ok(())
}

struct ProgramArgs {
    fname: String,
    // fpath: PathBuf,
    args: Vec<String>,
}

enum ShellCmd {
    Exit,
    Echo(Vec<String>),
    Pwd,
    Type(Vec<String>),
    Empty,
    Program(ProgramArgs),
    Unknown(String),
    Cd(String),
}

fn parse_cmd(cmd: &str) -> Result<ShellCmd> {
    let mut iter = cmd.split_whitespace();
    let first = iter.next();
    let rest: Vec<String> = iter.map(|x| x.to_string()).collect();
    let shell_cmd = match first {
        Some("cd") => {
            if rest.len() != 1 {
                bail!("Expecting only one argument for cd")
            }
            ShellCmd::Cd(rest[0].clone())
        }
        Some("exit") => ShellCmd::Exit,
        Some("echo") => ShellCmd::Echo(rest),
        Some("pwd") => {
            if !rest.is_empty() {
                bail!("pwd does not expect any arguments")
            }
            ShellCmd::Pwd
        }
        Some("type") => ShellCmd::Type(rest),
        // TODO: n2 (me) i want to put find_exec out of the parse.
        // all unknown should prb just be a program, and if it's isn't found, it will print the proper msg.
        Some(fname) => ShellCmd::Program(ProgramArgs {
            fname: fname.to_string(),
            args: rest,
        }),
        None => ShellCmd::Empty,
    };
    Ok(shell_cmd)
}

fn run_type(fnames: Vec<String>) -> Result<()> {
    for fname in fnames {
        if BUILTINS.contains(&fname.as_str()) {
            println!("{} is a shell builtin", fname);
            continue;
        }
        if let Some(p) = find_executable(&fname) {
            println!("{} is {}", fname, p.to_str().unwrap_or(""));
        } else {
            println!("{fname}: not found");
        }
    }
    Ok(())
}

fn run_program(p: ProgramArgs) -> Result<()> {
    let executable = find_executable(p.fname.as_str());
    match executable {
        Some(path) => {
            let exit_status = Command::new(path).arg0(p.fname).args(p.args).status()?;
            // for now, don't do anything with exit status
            Ok(())
        }
        None => {
            println!("{}: command not found", p.fname);
            Ok(())
        }
    }
}

fn find_executable(fname: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(fname);
        if candidate
            .metadata()
            .is_ok_and(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
        {
            return Some(candidate);
        }
    }
    None
}
