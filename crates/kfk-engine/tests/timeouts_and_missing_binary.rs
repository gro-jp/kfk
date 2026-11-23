mod common;
use common::*;

#[test]
fn timeouts_and_missing_binary() {
    let _g = ENV.lock().unwrap();
    let s = Store::temp();
    let v = vault(&s);
    std::env::set_var("KFK_CLAUDE", fake(&s.kh, "echo good > out.txt; sleep 30"));
    let t = task(&s, "slow agent", "x", "grep -q good out.txt", "");
    s.set_field(t.id, "timeout", "3").unwrap();
    s.claim(t.id).unwrap();
    run_task(&s, &v, t.id).unwrap();
    let r = &s.runs_of(t.id).unwrap()[0];
    assert!(r.note.starts_with("killed at its 3s limit; check passed"), "{}", r.note);
    assert_eq!(s.task(t.id).unwrap().status, "done"); // the work was there; the check decides

    std::env::set_var("KFK_CLAUDE", fake(&s.kh, "echo '{\"type\":\"result\",\"result\":\"ok\",\"usage\":{}}'"));
    s.set_cfg("check_timeout_s", "1").unwrap();
    let c = task(&s, "slow check", "x", "sleep 30", "");
    s.set_field(c.id, "max_attempts", "1").unwrap();
    s.claim(c.id).unwrap();
    run_task(&s, &v, c.id).unwrap();
    let r = &s.runs_of(c.id).unwrap()[0];
    assert_eq!(r.check_rc, Some(124));
    assert!(fs::read_to_string(format!("{}/check.log", r.dir)).unwrap().contains("past its time limit"));

    std::env::set_var("KFK_CLAUDE", "/nonexistent/claude");
    let m = task(&s, "no binary", "x", "true", "");
    s.claim(m.id).unwrap();
    run_task(&s, &v, m.id).unwrap();
    let m2 = s.task(m.id).unwrap();
    assert_eq!(m2.status, "blocked");
    assert!(m2.result.contains("cannot start /nonexistent/claude"), "{}", m2.result);
    assert_eq!(s.runs_of(m.id).unwrap()[0].status, "error");
    assert_eq!(ping(None).0, 2);
    std::env::remove_var("KFK_CLAUDE");
    assert_eq!(claude_bin(), "claude");
}
