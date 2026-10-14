use std::io::BufRead;
// Unix-only extension trait: adds `.mode()` to `Permissions`.
use std::os::unix::fs::PermissionsExt;

#[test]
fn explore_split_paths() {
    // `var_os` returns Option<OsString>: PATH isn't guaranteed to be UTF-8,
    // and `split_paths` takes anything AsRef<OsStr>, so no conversion needed.
    let path = std::env::var_os("PATH").expect("PATH not set");
    for dir in std::env::split_paths(&path) {
        println!("{}", dir.display());
        break;
    }

    // Edge cases: what does an empty entry (`::`) or a trailing `:` yield?
    for dir in std::env::split_paths("/usr/bin::/bin:") {
        println!("{dir:?}");
    }

    // Each item is a PathBuf, so `join` builds a candidate file path.
    // let candidate = std::path::Path::new("/usr/bin").join("ls");
    let candidate = std::path::Path::new("/bin").join("cat");
    println!("{candidate:?} exists: {}", candidate.exists());
    let mode = candidate
        .metadata()
        .ok()
        .map(|x| format!("{:o}", x.permissions().mode()));
    println!("{candidate:?} mode: {mode:?}");
}

#[test]
fn explore_is_executable() {
    // `mode()` is the raw st_mode from stat(2): file type bits + permission bits.
    // 0o111 masks the three execute bits (owner, group, other).
    // A directory, an executable, a plain file, a symlink, and a missing path:
    for p in ["/bin", "/bin/cat", "/etc/hosts", "/tmp", "/nope"] {
        let path = std::path::Path::new(p);
        match path.metadata() {
            Ok(meta) => {
                let mode = meta.permissions().mode();
                println!(
                    "{p:12} mode={mode:o} is_file={} any_x_bit={}",
                    meta.is_file(),
                    mode & 0o111 != 0
                );
            }
            Err(e) => println!("{p:12} error: {e}"),
        }
    }
}

#[test]
fn explore_read_line_at_eof() {
    let mut input: &[u8] = b"echo hi\n";
    let mut buf = String::new();

    let n = input.read_line(&mut buf).unwrap();
    println!("1st read: n={n}, buf={buf:?}");

    // Nothing left: is this an Err, or something else? And what happens to `buf`?
    let n = input.read_line(&mut buf).unwrap();
    println!("2nd read: n={n}, buf={buf:?}");
}

#[test]
fn explore_blank_line_vs_eof() {
    // A blank line, then a final line with no trailing newline (like typing
    // `exit` and hitting Ctrl-D instead of Enter), then nothing.
    let mut input: &[u8] = b"\nexit";
    for _ in 0..3 {
        let mut buf = String::new();
        let n = input.read_line(&mut buf).unwrap();
        println!("n={n}, raw={buf:?}, trimmed={:?}", buf.trim());
    }
}
