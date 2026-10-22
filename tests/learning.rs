use std::io::BufRead;
// Unix-only extension trait: adds `.mode()` to `Permissions`.
use std::os::unix::fs::PermissionsExt;

// MAYBE: n2 give me a learning test showing how to use combined pipe for a command in rust
#[test]
fn explore_chars_vs_bytes() {
    // A String is UTF-8 bytes. A `char` is one Unicode scalar value, which takes 1 to 4 bytes.
    // `bytes()` yields every u8; `chars()` decodes, so yes, it can consume several bytes per item.
    let s = "a é 🦀";
    println!(
        "len()={} (bytes), chars().count()={}",
        s.len(),
        s.chars().count()
    );
    println!("bytes: {:x?}", s.bytes().collect::<Vec<u8>>());

    // `char_indices` shows the byte offset where each char starts. Watch the gaps.
    for (i, c) in s.char_indices() {
        println!("byte {i}: {c:?} ({} bytes)", c.len_utf8());
    }

    // Accumulating: `collect::<String>()` from an iterator of chars...
    let no_spaces: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    println!("collected: {no_spaces:?}");

    // ...or `push` one char at a time when the loop has more going on (`push_str` for a &str).
    let mut out = String::new();
    for c in s.chars() {
        if c != ' ' {
            out.push(c);
        }
    }
    println!("pushed:    {out:?}");

    // Byte offsets and char counts are different units. `skip(n)` counts chars,
    // so it disagrees with slicing at byte offset n once there are multi-byte chars.
    // Here 'é' occupies bytes 2..4, so byte 3 is in the middle of it.
    // `s[3..]` would panic; `get` returns None instead.
    println!(
        "chars().skip(3): {:?}",
        s.chars().skip(3).collect::<String>()
    );
    println!("s.get(3..):      {:?}", s.get(3..));
    println!("s.get(5..):      {:?}", s.get(5..));
}

#[test]
fn explore_home() {
    use std::process::Command;

    // HOME is just an env var the login process set; the kernel doesn't know about it.
    // `var_os` -> None if unset (no UTF-8 check), `var` -> Err if unset or not UTF-8.
    println!("var_os(HOME): {:?}", std::env::var_os("HOME"));
    println!("var(HOME):    {:?}", std::env::var("HOME"));

    // `home_dir` reads HOME, and only if it's unset falls back to the passwd entry (getpwuid_r).
    println!("home_dir():   {:?}", std::env::home_dir());

    // `~` is shell syntax, not a path: the filesystem sees a dir literally named "~".
    println!(
        "Path(\"~\").exists(): {}",
        std::path::Path::new("~").exists()
    );

    // A shell expands `~` from *its own* HOME, so changing the child's env changes the result.
    let out = Command::new("/bin/sh")
        .args(["-c", "echo ~ ~/x"])
        .env("HOME", "/somewhere/else")
        .output()
        .unwrap();
    println!(
        "sh with HOME overridden: {}",
        String::from_utf8_lossy(&out.stdout).trim()
    );
}

#[test]
fn explore_cwd() {
    use std::process::Command;

    // The cwd is per-process kernel state (getcwd(3) / chdir(2)), not an env var.
    let original = std::env::current_dir().unwrap();
    println!("start:        {}", original.display());

    // chdir affects the whole process: every thread, so every parallel test too.
    // On macOS `/tmp` is a symlink. What does current_dir report afterwards?
    std::env::set_current_dir("/tmp").unwrap();
    println!(
        "after /tmp:   {}",
        std::env::current_dir().unwrap().display()
    );

    // Relative paths resolve against the current cwd.
    std::env::set_current_dir("..").unwrap();
    println!(
        "after ..:     {}",
        std::env::current_dir().unwrap().display()
    );

    // Failures are io::Errors; the kind tells you why.
    for bad in ["/nope", "/etc/hosts"] {
        match std::env::set_current_dir(bad) {
            Ok(()) => println!("unexpected: cd {bad} succeeded"),
            Err(e) => println!("cd {bad:11} -> Err(kind={:?}): {e}", e.kind()),
        }
    }

    // A child inherits our cwd across fork + exec...
    let out = Command::new("/bin/pwd").output().unwrap();
    println!(
        "child pwd:    {}",
        String::from_utf8_lossy(&out.stdout).trim()
    );

    // ...unless `Command::current_dir` sets it for that child only (chdir after fork, before exec).
    let out = Command::new("/bin/pwd")
        .current_dir("/usr")
        .output()
        .unwrap();
    println!(
        "child (/usr): {}",
        String::from_utf8_lossy(&out.stdout).trim()
    );
    println!(
        "ours still:   {}",
        std::env::current_dir().unwrap().display()
    );

    // Put it back so other tests in this process aren't affected.
    std::env::set_current_dir(&original).unwrap();
}

#[test]
fn explore_run_program() {
    use std::process::Command;

    // `status()` = fork + exec + wait. The child inherits our stdin/stdout/stderr,
    // so its output goes straight to the terminal; we only get the exit status back.
    let status = Command::new("/bin/echo")
        .args(["arg1", "arg2"])
        .status()
        .unwrap();
    println!(
        "status: {status:?}, success={}, code={:?}",
        status.success(),
        status.code()
    );

    // A non-zero exit is still Ok(status). It's not an Err.
    let status = Command::new("/usr/bin/false").status().unwrap();
    println!(
        "false -> success={}, code={:?}",
        status.success(),
        status.code()
    );

    // `output()` captures stdout/stderr into Vec<u8> instead of inheriting them.
    let out = Command::new("/bin/echo").arg("captured").output().unwrap();
    println!(
        "captured stdout: {:?}",
        String::from_utf8_lossy(&out.stdout)
    );

    // A bare name (no '/') makes Command do its own PATH search, like execvp.
    // A missing program shows up as Err from spawn, not as an exit status.
    match Command::new("definitely_not_a_program").status() {
        Ok(s) => println!("unexpected: {s:?}"),
        Err(e) => println!("missing program -> Err(kind={:?}): {e}", e.kind()),
    }
}

#[test]
fn explore_argv0() {
    use std::os::unix::process::CommandExt; // adds `.arg0()` on Unix
    use std::process::Command;

    // argv[0] is just a convention: whatever string the parent passes as the first
    // element of argv to execve(2). By default Command uses the program string as-is
    // (here the full path). `arg0` overrides it without changing which file gets run.
    // `ps -o args=` prints the child's argv as the kernel recorded it.
    for arg0 in [None, Some("my_sleep")] {
        let mut cmd = Command::new("/bin/sleep");
        cmd.arg("1");
        if let Some(name) = arg0 {
            cmd.arg0(name);
        }
        let mut child = cmd.spawn().unwrap(); // spawn = fork + exec, no wait
        let ps = Command::new("ps")
            .args(["-o", "args=", "-p", &child.id().to_string()])
            .output()
            .unwrap();
        println!(
            "arg0={arg0:?} -> ps sees: {}",
            String::from_utf8_lossy(&ps.stdout).trim()
        );
        child.wait().unwrap();
    }
}

#[test]
fn explore_split_paths() {
    // `var_os` returns Option<OsString>: PATH isn't guaranteed to be UTF-8,
    // and `split_paths` takes anything AsRef<OsStr>, so no conversion needed.
    let path = std::env::var_os("PATH").expect("PATH not set");
    for dir in std::env::split_paths(&path) {
        println!("{}", dir.display());
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
fn explore_access_x_ok() {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt; // adds `.as_bytes()` on OsStr

    // access(2) asks the kernel "could *I* (this process's real UID/GIDs) execute this?"
    // It applies the owner/group/other bits for you, unlike `mode & 0o111`.
    // Returns 0 on success, -1 on failure with the reason in errno.
    fn access_x(path: &std::path::Path) -> std::io::Result<()> {
        // C wants a NUL-terminated string; CString::new fails if the path contains a NUL byte.
        let c_path = CString::new(path.as_os_str().as_bytes())?;
        let rc = unsafe { libc::access(c_path.as_ptr(), libc::X_OK) };
        if rc == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error())
        }
    }

    // A file with only the group execute bit (mode 010): `any_x_bit` says yes,
    // but if we own it, only the owner bits apply to us, so access says no.
    let group_only = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("group_x_only");
    // Last run left it at mode 010, so the owner can't open it for writing. Unlinking only
    // needs write permission on the directory, not the file. Ignore NotFound on a first run.
    let _ = std::fs::remove_file(&group_only);
    std::fs::write(&group_only, "#!/bin/sh\necho hi\n").unwrap();
    std::fs::set_permissions(&group_only, std::fs::Permissions::from_mode(0o010)).unwrap();

    let paths = [
        std::path::Path::new("/bin/cat"),
        std::path::Path::new("/etc/hosts"),
        std::path::Path::new("/bin"), // X_OK on a directory means "can search it"
        std::path::Path::new("/nope"),
        group_only.as_path(),
    ];
    for path in paths {
        let any_x_bit = path
            .metadata()
            .is_ok_and(|m| m.permissions().mode() & 0o111 != 0);
        match access_x(path) {
            Ok(()) => println!("{:40} any_x_bit={any_x_bit:5} access=ok", path.display()),
            Err(e) => println!(
                "{:40} any_x_bit={any_x_bit:5} access={:?}",
                path.display(),
                e.kind()
            ),
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
