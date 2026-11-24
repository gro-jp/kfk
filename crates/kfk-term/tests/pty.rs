use kfk_term::Pty;
use std::io::{Read, Write};

/// read until `want` shows up or the program ends
fn read_until(r: &mut std::fs::File, want: &str) -> String {
    let (mut all, mut b) = (String::new(), [0u8; 4096]);
    while !all.contains(want) {
        match r.read(&mut b) {
            Ok(n) if n > 0 => all.push_str(&String::from_utf8_lossy(&b[..n])),
            _ => break,
        }
    }
    all
}

#[test]
fn a_program_on_a_pty_sees_a_terminal_of_the_given_size_and_what_is_typed() {
    let p = Pty::spawn("sh", &["-c".into(), "stty size; echo TERM=$TERM TMUX=${TMUX:-none}; read x; echo got:$x; stty size".into()], 90, 30).unwrap();
    let (mut r, mut w) = (p.reader().unwrap(), p.writer().unwrap());
    let first = read_until(&mut r, "TMUX=");
    assert!(first.contains("30 90") && first.contains("TERM=xterm-256color TMUX=none"), "{first}");
    p.resize(100, 40);
    w.write_all(b"hello\n").unwrap();
    let rest = read_until(&mut r, "40 100");
    assert!(rest.contains("got:hello") && rest.contains("40 100"), "{rest}");
}

#[test]
fn dropping_the_pty_ends_the_program_and_a_missing_program_is_an_error() {
    let p = Pty::spawn("sh", &["-c".into(), "echo $$; sleep 600".into()], 80, 24).unwrap();
    let mut r = p.reader().unwrap();
    let pid: i32 = read_until(&mut r, "\n").trim().parse().unwrap();
    drop(p);
    assert!(unsafe { libc::kill(pid, 0) } != 0);
    let e = Pty::spawn("/no/such/program", &[], 80, 24).err().unwrap().to_string();
    assert!(e.contains("cannot run /no/such/program"), "{e}");
}
