use kfk_mizan::*;
use kfk_store::Store;

fn line(ts: &str, mid: &str, out: i64) -> String {
    format!(r#"{{"type":"assistant","timestamp":"{ts}","message":{{"id":"{mid}","usage":{{"input_tokens":10,"output_tokens":{out}}}}}}}"#)
}

#[test]
fn node_spend_follows_the_transcripts() {
    let s = Store::temp();
    let proj = format!("{}/projects", s.kh);
    std::fs::create_dir_all(format!("{proj}/-some-dir")).unwrap();
    std::fs::create_dir_all(format!("{proj}/-other")).unwrap();
    s.set_cfg("claude_projects", &proj).unwrap();
    assert_eq!(transcript(std::path::Path::new("/nonexistent"), "x"), None);
    assert_eq!(transcript(std::path::Path::new(&proj), "x"), None);
    assert!(appended("/nonexistent", 0).is_err());

    // cos is up on session A; an orchestrator's session B has no transcript yet
    s.put_seat("cos", "cos", "fable", "high").unwrap();
    s.seat_slot("cos", 2).unwrap();
    s.seat_started("cos", "A", 1).unwrap();
    s.put_seat("o1", "orch", "opus", "high").unwrap();
    s.seat_started("o1", "B", 2).unwrap();
    let path = format!("{proj}/-some-dir/A.jsonl");
    std::fs::write(&path, format!("{}\n{}", line("1970-01-01T05:00:00Z", "m1", 1), &line("1970-01-01T05:00:01Z", "m2", 2)[..40])).unwrap();
    assert_eq!(fold_sessions(&s).unwrap(), 1); // sessions are opened from the seats; the partial last line waits
    let a = s.session("A").unwrap().unwrap();
    assert_eq!((a.seat.as_str(), a.slot, a.path.as_str()), ("cos", Some(2), path.as_str()));
    assert_eq!(s.spend_of("A").unwrap().out_tok, 1);
    assert_eq!(s.session("B").unwrap().unwrap().slot, Some(kfk_usage::LOGIN));
    assert_eq!(fold_sessions(&s).unwrap(), 0); // nothing new

    // the file grows: the rest of m2, and m1 again at a later usage
    std::fs::write(&path, format!("{}\n{}\n{}\n", line("1970-01-01T05:00:00Z", "m1", 1), line("1970-01-01T05:00:01Z", "m2", 2), line("1970-01-01T06:00:00Z", "m2", 5))).unwrap();
    assert_eq!(fold_sessions(&s).unwrap(), 2);
    let sp = s.spend_of("A").unwrap();
    assert_eq!((sp.in_tok, sp.out_tok, sp.turns), (20, 6, 2));
    assert_eq!(appended(&path, std::fs::metadata(&path).unwrap().len() as i64).unwrap(), "");

    // cos gets a fresh session: A is closed after a last read, C is opened; a stopped node's session closes
    s.seat_started("cos", "C", 3).unwrap();
    s.seat_state("o1", "stopped").unwrap();
    fold_sessions(&s).unwrap();
    assert!(s.session("A").unwrap().unwrap().ended.is_some());
    assert!(s.session("B").unwrap().unwrap().ended.is_some());
    assert!(s.session("C").unwrap().unwrap().ended.is_none());
    assert_eq!(s.sessions(true).unwrap().len(), 1);
}
