use std::io::BufRead;

// `&[u8]` implements `BufRead`, so a byte string can stand in for stdin.
// `Stdin::read_line` has the same contract as `BufRead::read_line`.

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
