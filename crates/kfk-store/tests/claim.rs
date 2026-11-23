mod common;
use common::*;

#[test]
fn claim_is_exclusive_and_needs_a_check() {
    let s = temp();
    let a = s.add_task(&t("x", None)).unwrap();
    let other = Store::open(&s.kh).unwrap();
    assert!(s.claim(a).unwrap());
    assert!(!other.claim(a).unwrap());
    let b = s.add_task(&Task { chk: " ".into(), ..t("no check", None) }).unwrap();
    assert!(s.claim(b).is_err());
    assert!(s.claim(99).is_err());
    s.set_runner(a, 77).unwrap();
    assert_eq!(s.task(a).unwrap().runner, Some(77));
    s.finish(a, "failed", "nope").unwrap();
    s.c.execute("update tasks set attempts=3 where id=?1", [a]).unwrap();
    assert!(s.claim(a).unwrap()); // a failed task can be run again, from attempt 0
    assert_eq!(s.task(a).unwrap().attempts, 0);
    s.finish(a, "done", "ok").unwrap();
    s.finish(a, "failed", "late").unwrap(); // no longer running: ignored
    assert_eq!(s.task(a).unwrap().status, "done");
    assert!(!s.claim(a).unwrap());
}

#[test]
fn reconcile_requeues_orphans() {
    let s = temp();
    let a = s.add_task(&t("dead runner", None)).unwrap();
    let b = s.add_task(&t("live runner", None)).unwrap();
    for id in [a, b] {
        s.claim(id).unwrap();
        s.set_runner(id, if id == a { 111 } else { 222 }).unwrap();
        let Gate::Started { run, .. } = s.begin_run(id, 1, "opus", "high", &[]).unwrap() else { panic!() };
        s.run_pid(run, if id == a { 333 } else { 444 }).unwrap();
    }
    let alive = |p: i64| p == 222 || p == 333; // a's runner is dead but its agent still lives
    assert_eq!(s.reconcile(&alive).unwrap(), (1, vec![333]));
    let ta = s.task(a).unwrap();
    assert_eq!((ta.status.as_str(), ta.attempts, ta.runner), ("todo", 0, None));
    assert_eq!(s.runs_of(a).unwrap()[0].status, "lost");
    assert_eq!(s.task(b).unwrap().status, "running");
    assert_eq!(s.reconcile(&alive).unwrap(), (0, vec![]));
    // a dead runner whose agent is gone too: requeued, nothing to stop
    let c = s.add_task(&t("both dead", None)).unwrap();
    s.claim(c).unwrap();
    s.set_runner(c, 555).unwrap();
    let Gate::Started { run, .. } = s.begin_run(c, 1, "opus", "high", &[]).unwrap() else { panic!() };
    s.run_pid(run, 666).unwrap();
    assert_eq!(s.reconcile(&alive).unwrap(), (1, vec![]));
}
