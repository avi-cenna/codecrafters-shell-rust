use std::fs::File;
use crate::parser::OutputType;
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

pub fn run_echo(args: &[String], redirect: &OutputType) -> Result<()> {
    let mut out = output_buffer(redirect)?;
    writeln!(out, "{}", args.join(" "))?;
    Ok(())
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

pub fn run_pwd(args: &[String], redirect: &OutputType) -> Result<()> {
    if !args.is_empty() {
        bail!("expected no arguments")
    }
    let mut out = output_buffer(redirect)?;
    let cwd = std::env::current_dir()?;
    writeln!(out, "{}", cwd.display())?;
    Ok(())
}

// TODO: n2 pls review
pub fn run_type(args: &[String], redirect: &OutputType) -> Result<()> {
    let mut out = output_buffer(redirect)?;
    for arg in args {
        if BuiltinCmd::from_name(arg).is_some() {
            writeln!(out, "{arg} is a shell builtin")?;
        } else if let Some(p) = find_executable(arg) {
            writeln!(out, "{} is {}", arg, p.display())?;
        } else {
            eprintln!("{arg}: not found");
        }
    }
    Ok(())
}

pub fn run_program(name: &str, args: &[String], output_type: &OutputType) -> Result<()> {
    // dbg!(name, args); // only for manual inspection
    let stdio = match output_type {
        OutputType::Stdout => Stdio::inherit(),
        OutputType::File(fpath) => {
            let file = File::create(fpath)?;
            Stdio::try_from(file)?
        }
    };
    if let Some(path) = find_executable(name) {
        let _ = Command::new(path).arg0(name).args(args).stdout(stdio).status()?;
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

fn output_buffer(ot: &OutputType) -> Result<Box<dyn Write>> {
    let buf: Box<dyn Write> = match ot {
        OutputType::File(f) => Box::new(std::fs::File::create(f)?),
        OutputType::Stdout => Box::new(std::io::stdout()),
    };
    Ok(buf)
}
