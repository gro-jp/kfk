mod common;
use common::*;
use std::process::Command;
use std::os::unix::process::CommandExt;

#[test]
fn a_group_that_ignores_term_is_killed() {
    let mut p = Command::new("bash").args(["-c", "trap '' TERM; sleep 60 & wait"]).process_group(0).spawn().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(300));
    let t = std::time::Instant::now();
    kill_group(&mut p, 1);
    assert!(p.try_wait().unwrap().is_some());
    assert!(t.elapsed().as_secs() < 5);
    let mut q = Command::new("sleep").arg("60").process_group(0).spawn().unwrap();
    kill_group(&mut q, 5); // stops at TERM
    assert!(q.try_wait().unwrap().is_some());
    assert_eq!(wait_timeout(&mut Command::new("true").spawn().unwrap(), 5), Some(0));
}

#[test]
fn a_detached_child_logs_under_the_home() {
    let s = Store::temp();
    let mut c = spawn_detached(&s.kh, &["--list".into()], "self").unwrap(); // this test binary, listing its tests
    assert!(c.wait().unwrap().success());
    assert!(fs::read_to_string(format!("{}/log/self.log", s.kh)).unwrap().contains("a_detached_child_logs_under_the_home"));
}
