use anyhow::Result;
use std::fs;
use std::fs::FileType;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub struct PathInfo {
    pub path: PathBuf,
    pub file_name: String,
    pub file_type: FileType,
}

pub fn find_executable(name: &str) -> Option<PathBuf> {
    if name.contains('/') {
        let candidate = PathBuf::from(name);
        if is_executable(&candidate) {
            Some(candidate)
        } else {
            None
        }
    } else {
        let path = std::env::var_os("PATH")?;
        let mut dirs = std::env::split_paths(&path);
        dirs.find(|dir| {
            let candidate = dir.join(name);
            is_executable(&candidate)
        })
        .map(|dir| dir.join(name))
    }
}

pub fn get_all_paths_in_dir<P: AsRef<Path>>(dir: P) -> Result<Vec<PathInfo>> {
    let path_infos: Vec<PathInfo> = fs::read_dir(dir)?
        .flatten() // Flatten to get all DirEntry Results
        .filter_map(|e| {
            let file_name = e.file_name().display().to_string();
            let file_type = e.file_type().ok()?;
            let path = e.path();
            Some(PathInfo {
                path,
                file_name,
                file_type,
            })
        })
        .collect();
    Ok(path_infos)
}

pub fn is_executable(f: &Path) -> bool {
    f.metadata()
        .is_ok_and(|m| m.is_file() && (m.permissions().mode() & 0o111 != 0))
}
