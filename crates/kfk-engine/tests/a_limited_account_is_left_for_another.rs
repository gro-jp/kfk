mod common;
use common::*;

#[test]
fn a_limited_account_is_left_for_another() {
    let _g = ENV.lock().unwrap();
    let s = Store::temp();
    std::env::set_var("KFK_CLAUDE", fake(&s.kh, LIMITED));
    let v = vault(&s);
    for (slot, tok) in [(0, "tok0"), (1, "tok1")] {
        v.put(slot, tok, "a@b").unwrap();
        s.put_account(slot, "a@b", "").unwrap();
    }
    let w: Wins = [("seven_day".to_string(), Win { util: Some(50.0), ..Default::default() })].into_iter().collect();
    s.record_windows(1, &w, "sonnet").unwrap(); // slot 0 looks emptier, so it is tried first
    let t = task(&s, "x", "x", "grep -q good out.txt", "");
    s.claim(t.id).unwrap();
    run_task(&s, &v, t.id).unwrap();
    let runs = s.runs_of(t.id).unwrap();
    assert_eq!(runs.iter().map(|r| (r.slot, r.status.as_str())).collect::<Vec<_>>(), [(0, "limit"), (1, "pass")]);
    let done = s.task(t.id).unwrap();
    assert_eq!((done.status.as_str(), done.attempts), ("done", 1)); // the limited try did not count
    assert_eq!(s.account(0).unwrap().unwrap().wins["blocked@sonnet"].status.as_deref(), Some("rejected"));
    // with every account full the task is blocked, not failed
    s.record_windows(1, &[("seven_day".to_string(), Win { util: Some(99.0), reset: Some(now() + 500.0), status: None })].into_iter().collect(), "sonnet").unwrap();
    let b = task(&s, "y", "x", "true", "");
    s.claim(b.id).unwrap();
    run_task(&s, &v, b.id).unwrap();
    let b = s.task(b.id).unwrap();
    assert_eq!(b.status, "blocked");
    assert!(b.result.starts_with("no account has headroom for sonnet; earliest reset "), "{}", b.result);
    std::env::remove_var("KFK_CLAUDE");
}
