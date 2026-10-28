use crate::parser::ShellCmd;
use anyhow::{Result, bail};
use std::fs;
use std::fs::DirEntry;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::prelude::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

pub fn all_executables_on_path() -> Result<Vec<DirEntry>> {
    let Some(path) = std::env::var_os("PATH") else {
        bail!("Failed to get PATH variable")
    };
    let dirs = std::env::split_paths(&path);
    let entries: Vec<DirEntry> = dirs
        .filter_map(|x| fs::read_dir(x).ok())
        .flatten() // Flatten to get all DirEntry Results
        .filter_map(Result::ok)
        .filter(|p| is_executable(&p.path()))
        .collect();
    Ok(entries)
}

pub fn is_executable(f: &Path) -> bool {
    f.metadata()
        .is_ok_and(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
}
