use crate::parser::ShellCmd;
use anyhow::{Context, Result, bail};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::prelude::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub enum BuiltinCmd {
    Cd,
    Echo,
    Exit,
    Pwd,
    Type,
}

impl BuiltinCmd {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "cd" => Some(Self::Cd),
            "exit" => Some(Self::Exit),
            "echo" => Some(Self::Echo),
            "pwd" => Some(Self::Pwd),
            "type" => Some(Self::Type),
            _ => None,
        }
    }
}

pub fn run_echo(cmd: &ShellCmd, out: &mut dyn Write) -> Result<()> {
    writeln!(out, "{}", cmd.args.join(" "))?;
    Ok(())
}

pub fn run_cd(cmd: &ShellCmd) -> Result<()> {
    let [arg] = cmd.args.as_slice() else {
        bail!("expected exactly one argument")
    };
    let target = if arg == "~" {
        std::env::home_dir().context("Cannot get home directory")?
    } else {
        PathBuf::from(arg)
    };
    if !target.is_dir() {
        // CodeCrafters expects this message
        bail!("{}: No such file or directory", target.display())
    }
    std::env::set_current_dir(&target)?;
    Ok(())
}

pub fn run_pwd(cmd: &ShellCmd, out: &mut dyn Write) -> Result<()> {
    if !&cmd.args.is_empty() {
        bail!("expected no arguments")
    }
    let cwd = std::env::current_dir()?;
    writeln!(out, "{}", cwd.display())?;
    Ok(())
}

pub fn run_type(cmd: &ShellCmd, out: &mut dyn Write, err: &mut dyn Write) -> Result<()> {
    for arg in &cmd.args {
        if BuiltinCmd::from_name(arg).is_some() {
            writeln!(out, "{arg} is a shell builtin")?;
        } else if let Some(p) = find_executable(arg) {
            writeln!(out, "{} is {}", arg, p.display())?;
        } else {
            writeln!(err, "{arg}: not found")?;
        }
    }
    Ok(())
}

pub fn run_program(cmd: &ShellCmd, out: Stdio, err: Stdio) -> Result<()> {
    if let Some(path) = find_executable(&cmd.name) {
        let _ = Command::new(path)
            .arg0(&cmd.name)
            .args(&cmd.args)
            .stdout(out)
            .stderr(err)
            .status()?;
        // for now, don't do anything with exit status
    } else {
        bail!("command not found");
    }
    Ok(())
}

fn find_executable(fname: &str) -> Option<PathBuf> {
    if fname.contains('/') {
        let candidate = PathBuf::from(fname);
        if is_executable(&candidate) {
            Some(candidate)
        } else {
            None
        }
    } else {
        let path = std::env::var_os("PATH")?;
        let mut dirs = std::env::split_paths(&path);
        dirs.find(|dir| {
            let candidate = dir.join(fname);
            is_executable(&candidate)
        })
        .map(|dir| dir.join(fname))
    }
}

fn is_executable(f: &Path) -> bool {
    f.metadata()
        .is_ok_and(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
}
