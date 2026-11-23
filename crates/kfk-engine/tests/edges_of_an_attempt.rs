mod common;
use common::*;

#[test]
fn edges_of_an_attempt() {
    let _g = ENV.lock().unwrap();
    let s = Store::temp();
    let v = vault(&s);
    // the stream itself names the full window: no extra block is recorded
    std::env::set_var("KFK_CLAUDE", fake(&s.kh, r#"echo '{"type":"rate_limit_event","rate_limit_info":{"status":"rejected","rateLimitType":"seven_day","utilization":1.0,"resetsAt":1999999999}}'; echo '{"type":"result","is_error":true,"result":"limit"}'; exit 1"#));
    v.put(0, "tok0", "a@b").unwrap();
    s.put_account(0, "a@b", "").unwrap();
    let t = task(&s, "x", "x", "true", "");
    s.claim(t.id).unwrap();
    run_task(&s, &v, t.id).unwrap();
    let w = s.account(0).unwrap().unwrap().wins;
    assert_eq!(w["seven_day"].status.as_deref(), Some("rejected"));
    assert!(!w.keys().any(|k| k.starts_with("blocked")));
    assert_eq!(s.task(t.id).unwrap().status, "blocked");
    s.del_account(0).unwrap();

    // a check that cannot even start is an error, not a verdict
    assert!(check(&s, &t, "/nonexistent-dir", &format!("{}/c.log", s.kh)).is_err());

    // a failed earlier run without a check log adds nothing to the prompt; with one, its tail
    let p = task(&s, "p", "the spec", "true", "");
    for (i, log) in [(1, None), (2, Some("boom on line 7"))] {
        let kfk_store::Gate::Started { run, .. } = s.begin_run(p.id, i, "sonnet", "medium", &[]).unwrap() else { panic!() };
        let dir = format!("{}/runs/{run}", s.kh);
        fs::create_dir_all(&dir).unwrap();
        if let Some(l) = log {
            fs::write(format!("{dir}/check.log"), l).unwrap();
        }
        s.run_place(run, &dir, "/w").unwrap();
        s.run_end(run, "fail", Some(1), "").unwrap();
        let text = prompt(&s, &p, run + 1, "/w");
        assert!(text.contains("the spec") && text.contains("# Task T"));
        assert_eq!(text.contains("previous attempt"), log.is_some());
        assert_eq!(text.contains("boom on line 7"), log.is_some());
    }
    std::env::remove_var("KFK_CLAUDE");
}

#[test]
fn a_full_house_makes_the_runner_wait() {
    let _g = ENV.lock().unwrap();
    let s = Store::temp();
    let v = vault(&s);
    std::env::set_var("KFK_CLAUDE", fake(&s.kh, "echo good > out.txt; echo '{\"type\":\"result\",\"result\":\"ok\",\"usage\":{}}'"));
    s.set_cfg("max_agents", "1").unwrap();
    let hog = task(&s, "hog", "x", "true", "");
    let kfk_store::Gate::Started { run, .. } = s.begin_run(hog.id, 1, "sonnet", "medium", &[]).unwrap() else { panic!() };
    let kh = s.kh.clone();
    let free = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        Store::open(&kh).unwrap().run_end(run, "pass", Some(0), "").unwrap();
    });
    let t = task(&s, "waits its turn", "x", "grep -q good out.txt", "");
    s.claim(t.id).unwrap();
    let t0 = std::time::Instant::now();
    run_task(&s, &v, t.id).unwrap();
    free.join().unwrap();
    assert!(t0.elapsed().as_millis() >= 1400);
    assert_eq!(s.task(t.id).unwrap().status, "done");
    std::env::remove_var("KFK_CLAUDE");
}

#[test]
fn a_runner_can_wait_for_a_window_to_reset() {
    let _g = ENV.lock().unwrap();
    let s = Store::temp();
    let v = vault(&s);
    std::env::set_var("KFK_CLAUDE", fake(&s.kh, "echo good > out.txt; echo '{\"type\":\"result\",\"result\":\"ok\",\"usage\":{}}'"));
    v.put(0, "tok0", "a@b").unwrap();
    s.put_account(0, "a@b", "").unwrap();
    let full = |secs: f64| -> Wins { [("seven_day".to_string(), Win { util: Some(99.0), reset: Some(now() + secs), status: None })].into_iter().collect() };
    s.record_windows(0, &full(3.0), "sonnet").unwrap();
    s.set_cfg("capacity_wait_s", "30").unwrap();
    let t = task(&s, "waits for the reset", "x", "grep -q good out.txt", "");
    s.claim(t.id).unwrap();
    let kh = s.kh.clone();
    let id = t.id;
    let peek = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        Store::open(&kh).unwrap().task(id).unwrap()
    });
    run_task(&s, &v, t.id).unwrap();
    let mid = peek.join().unwrap();
    assert!(mid.status == "running" && mid.result.starts_with("waiting: no account has headroom for sonnet"), "{} {}", mid.status, mid.result);
    let done = s.task(t.id).unwrap();
    assert_eq!((done.status.as_str(), done.attempts), ("done", 1));
    // a wait that runs out blocks the task
    s.record_windows(0, &full(600.0), "sonnet").unwrap();
    s.set_cfg("capacity_wait_s", "1").unwrap();
    let b = task(&s, "gives up", "x", "true", "");
    s.claim(b.id).unwrap();
    run_task(&s, &v, b.id).unwrap();
    let b = s.task(b.id).unwrap();
    assert!(b.status == "blocked" && b.result.starts_with("no account has headroom"), "{}", b.result);
    std::env::remove_var("KFK_CLAUDE");
}
