mod common;
use common::*;

#[test]
fn gate_caps_agents_and_needs_headroom() {
    let s = temp();
    s.set_cfg("max_agents", "1").unwrap();
    let a = s.add_task(&t("x", None)).unwrap();
    let Gate::Started { run, slot } = s.begin_run(a, 1, "opus", "high", &[]).unwrap() else { panic!() };
    assert_eq!(slot, LOGIN);
    assert_eq!(s.begin_run(a, 2, "opus", "high", &[]).unwrap(), Gate::Full);
    s.run_place(run, "/d", "/w").unwrap();
    s.run_pid(run, 55).unwrap();
    s.run_usage(run, &kfk_stream::Parsed { input: 10, cw: 100, cr: 1000, out: 50, cost: 0.5, turns: 4, ..Default::default() }).unwrap();
    s.run_end(run, "fail", Some(1), "check exit 1").unwrap();
    let r = s.run(run).unwrap();
    assert_eq!((r.dir.as_str(), r.wd.as_str(), r.pid, r.weighted(), r.turns, r.check_rc, r.status.as_str()), ("/d", "/w", Some(55), 485, 4, Some(1), "fail"));
    assert!(r.ended.is_some() && r.cost == 0.5 && r.note == "check exit 1" && r.attempt == 1 && r.task == a && r.effort == "high" && r.model == "opus");
    assert_eq!(s.last_failed_dir(a, run + 1).unwrap().as_deref(), Some("/d"));
    assert_eq!(s.last_failed_dir(a, run).unwrap(), None);
    assert_eq!(s.task(a).unwrap().attempts, 1);
    s.unattempt(a).unwrap();
    s.unattempt(a).unwrap();
    assert_eq!(s.task(a).unwrap().attempts, 0);
    s.put_account(0, "a@b", "").unwrap();
    s.record_windows(0, &win(99.0), "opus").unwrap();
    assert_eq!(s.begin_run(a, 1, "opus", "high", &[]).unwrap(), Gate::NoAccount);
    assert_eq!(s.runs_of(a).unwrap().len(), 1);
    assert_eq!(s.runs_since(0.0).unwrap().len(), 1);
    assert!(s.running().unwrap().is_empty());
    s.c.execute_batch("drop table runs").unwrap(); // a failing statement inside the gate rolls back and reports
    assert!(s.begin_run(a, 1, "opus", "high", &[]).is_err());
    assert!(s.c.execute_batch("begin immediate; commit").is_ok());
}
