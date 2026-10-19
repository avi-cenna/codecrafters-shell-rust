use crate::parser::{ShellCmd, parse_cmd_name};
use anyhow::{Result, bail};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::prelude::CommandExt;
use std::path::PathBuf;
use std::process::Command;

pub fn run_echo(args: &[String]) {
    println!("{}", args.join(" "));
}

/// TODO: n2 is this idiomatic? any way i could improve it, or improve variable naming?
pub fn run_cd(args: &[String]) -> Result<()> {
    if args.len() != 1 {
        bail!("expected exactly one argument")
    }
    let path = args[0].as_str();
    let candidate = if path == "~" {
        match std::env::var_os("HOME") {
            Some(s) => PathBuf::from(s),
            None => bail!("Home directory is unset"),
        }
    } else {
        PathBuf::from(path)
    };
    if !candidate.is_dir() {
        bail!("{}: No such file or directory", candidate.display())
    }
    std::env::set_current_dir(&candidate)?;
    Ok(())
}

pub fn run_pwd(args: &[String]) -> Result<()> {
    if !args.is_empty() {
        bail!("expected no arguments")
    }
    let cwd = std::env::current_dir()?;
    println!("{}", cwd.display());
    Ok(())
}

/// TODO: n2 please review this. is it idiomatic?
// pub fn run_type(fnames: Vec<String>) { // try one version with borrowing
pub fn run_type(fnames: Vec<String>) {
    for fname in fnames {
        let shell_cmd = parse_cmd_name(fname.as_str());
        match shell_cmd {
            ShellCmd::Builtin(_) => println!("{fname} is a shell builtin"),
            ShellCmd::Program(_) => {
                if let Some(p) = find_executable(&fname) {
                    println!("{} is {}", fname, p.display());
                } else {
                    println!("{fname}: not found");
                }
            }
        }
        // if BUILTINS.contains(&fname.as_str()) {}
    }
}

pub fn run_program(name: String, args: Vec<String>) -> Result<()> {
    let executable = find_executable(name.as_str());
    if let Some(path) = executable {
        let _exit_status = Command::new(path).arg0(name).args(args).status()?;
        // for now, don't do anything with exit status
        // Ok(())
    } else {
        println!("{name}: command not found");
    }
    Ok(())
}

pub fn find_executable(fname: &str) -> Option<PathBuf> {
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
