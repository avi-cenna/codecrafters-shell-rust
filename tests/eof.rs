use std::io::Write;
use std::process::{Command, ExitStatus, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime};

#[test]
fn exit_on_eof() {
    let mut sh = Command::new(env!("CARGO_BIN_EXE_codecrafters-shell"))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    sh.stdin.take().unwrap().write_all(b"echo hi\n").unwrap();

    let deadline = Instant::now() + Duration::from_secs(2);

    while Instant::now() < deadline {
        match sh.try_wait().expect("Error attempting to wait") {
            Some(status) => {
                assert_eq!(status.code(), Some(0));
                return
            }
            None => sleep(Duration::from_millis(10)),
        }
    }
    sh.kill().expect("Process should be killed");
    sh.wait().expect("Child should not be running");
    panic!("Process hangs for over 2 seconds!")
}
