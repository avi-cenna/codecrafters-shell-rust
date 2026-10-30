use crate::parser::ShellCmd;
use anyhow::{Result, bail};
use std::fs;
use std::fs::{DirEntry, FileType};
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

pub fn all_executables_on_path() -> Result<Vec<PathInfo>> {
    let Some(path) = std::env::var_os("PATH") else {
        bail!("Failed to get PATH variable")
    };
    let dirs = std::env::split_paths(&path);
    let entries: Vec<PathInfo> = dirs
        .filter_map(|x| all_paths_in_dir(&x).ok())
        .flatten() 
        .filter(|p| is_executable(&p.path))
        .collect();
    Ok(entries)
}

pub struct PathInfo {
    pub path: PathBuf,
    pub file_name: String,
    pub file_type: FileType,
}

pub fn all_paths_in_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<PathInfo>> {
    let path_infos: Vec<PathInfo> = fs::read_dir(dir)?
        .flatten() // Flatten to get all DirEntry Results
        .filter_map(|e| {
            let file_name = e.file_name().display().to_string();
            let file_type = e.file_type();
            let path = e.path();
            match file_type {
                Ok(file_type) => Some(PathInfo {
                    path,
                    file_name,
                    file_type,
                }),
                Err(_) => None,
            }
        })
        .collect();
    Ok(path_infos)
}

pub fn is_executable(f: &Path) -> bool {
    f.metadata()
        .is_ok_and(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
}
