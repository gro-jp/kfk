use std::collections::HashMap;
use kfk_usage::*;


fn w(util: Option<f64>, reset: Option<f64>, status: Option<&str>) -> Win {
    Win { util, reset, status: status.map(String::from) }
}
fn wins(v: &[(&str, Win)]) -> Wins {
    v.iter().map(|(k, x)| (k.to_string(), x.clone())).collect()
}

#[test]
fn family() {
    assert_eq!(fam("claude-fable-5-2"), "fable");
    assert_eq!(fam("opus"), "opus");
    assert_eq!(fam("mystery"), "mystery");
}

#[test]
fn merge_scopes_model_windows() {
    let mut cur = Wins::new();
    merge(&mut cur, &wins(&[("seven_day", w(Some(10.0), None, None)), ("seven_day_oi", w(Some(90.0), None, None)), ("x@opus", w(Some(1.0), None, None))]), "claude-fable-5-2");
    assert!(cur.contains_key("seven_day") && cur.contains_key("seven_day_oi@fable") && cur.contains_key("x@opus"));
}

#[test]
fn util_picks_worst_live_window_in_scope() {
    let ws = wins(&[
        ("seven_day", w(Some(40.0), Some(2000.0), None)),
        ("five_hour", w(Some(99.0), Some(500.0), None)),        // expired
        ("seven_day_oi@fable", w(Some(80.0), Some(3000.0), None)),
        ("blocked@opus", w(Some(5.0), Some(3000.0), Some("rejected"))),
        ("odd", w(None, None, None)),
    ]);
    assert_eq!(util(&ws, "fable", 1000.0), (Some(80.0), Some(3000.0)));
    assert_eq!(util(&ws, "opus", 1000.0), (Some(100.0), Some(3000.0)));
    assert_eq!(util(&ws, "sonnet", 1000.0), (Some(40.0), Some(2000.0)));
    assert_eq!(util(&Wins::new(), "sonnet", 1000.0), (None, None));
}

#[test]
fn pick_prefers_headroom_then_load() {
    let c = |slot, u: f64, off| Cand { slot, off, wins: wins(&[("seven_day", w(Some(u), None, None))]) };
    let none = HashMap::new();
    assert_eq!(pick(&[], "opus", &[], 97.0, &none, 0.0), Some(LOGIN));
    assert_eq!(pick(&[c(0, 50.0, true)], "opus", &[], 97.0, &none, 0.0), Some(LOGIN));
    let pool = [c(0, 60.0, false), c(1, 20.0, false), c(2, 5.0, true), c(3, 98.0, false)];
    assert_eq!(pick(&pool, "opus", &[], 97.0, &none, 0.0), Some(1));
    assert_eq!(pick(&pool, "opus", &[1], 97.0, &none, 0.0), Some(0));
    assert_eq!(pick(&pool, "opus", &[0, 1], 97.0, &none, 0.0), None);
    let load: HashMap<i64, f64> = [(1, 50.0)].into_iter().collect();
    assert_eq!(pick(&pool, "opus", &[], 97.0, &load, 0.0), Some(0));
    let unknown = [Cand { slot: 7, off: false, wins: Wins::new() }, c(8, 40.0, false)];
    assert_eq!(pick(&unknown, "opus", &[], 97.0, &none, 0.0), Some(7));
}

#[test]
fn formats() {
    assert_eq!(fmt(&Wins::new(), 0.0), "-");
    let ws = wins(&[("seven_day", w(Some(41.6), None, None)), ("five_hour", w(None, None, None)), ("b@opus", w(None, None, Some("rejected"))), ("old", w(Some(1.0), Some(1.0), None))]);
    assert_eq!(fmt(&ws, 10.0), "b@opus FULL, 5h ?, wk 41%");
    assert_eq!(parse("not json"), Wins::new());
    assert_eq!(parse(r#"{"a":{"util":1.0,"reset":null,"status":null}}"#).len(), 1);
}
