use anyhow::{Context, Result, bail};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::prelude::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

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

pub fn run_echo(args: &[String]) {
    println!("{}", args.join(" "));
}

pub fn run_cd(args: &[String]) -> Result<()> {
    let [arg] = args else {
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

pub fn run_pwd(args: &[String]) -> Result<()> {
    if !args.is_empty() {
        bail!("expected no arguments")
    }
    let cwd = std::env::current_dir()?;
    println!("{}", cwd.display());
    Ok(())
}

pub fn run_type(fnames: &[String]) {
    for fname in fnames {
        if BuiltinCmd::from_name(fname).is_some() {
            println!("{fname} is a shell builtin");
        } else if let Some(p) = find_executable(fname) {
            println!("{} is {}", fname, p.display());
        } else {
            println!("{fname}: not found");
        }
    }
}

pub fn run_program(name: &str, args: &[String]) -> Result<()> {
    // dbg!(name, args); // only for manual inspection
    if let Some(path) = find_executable(name) {
        let _ = Command::new(path).arg0(name).args(args).status()?;
        // for now, don't do anything with exit status
    } else {
        bail!("command not found");
    }
    Ok(())
}

pub fn find_executable(fname: &str) -> Option<PathBuf> {
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
