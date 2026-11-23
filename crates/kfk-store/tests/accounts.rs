mod common;
use common::*;

#[test]
fn accounts_and_windows() {
    let s = temp();
    assert!(s.pool().unwrap().is_empty());
    assert_eq!(s.account(LOGIN).unwrap().unwrap().label, "saved-login");
    s.put_account(2, "a@b", "2026-11-22").unwrap();
    s.record_windows(2, &win(40.0), "opus").unwrap();
    s.record_windows(2, &Wins::new(), "opus").unwrap();
    s.record_windows(77, &win(1.0), "opus").unwrap(); // unknown slot: ignored
    let a = s.account(2).unwrap().unwrap();
    assert_eq!((a.label.as_str(), a.minted.as_str(), a.off), ("a@b", "2026-11-22", false));
    assert_eq!(a.wins["seven_day"].util, Some(40.0));
    assert!(a.asof.is_some());
    assert!(s.set_off(2, true).unwrap() && !s.set_off(9, true).unwrap() && !s.set_off(LOGIN, true).unwrap());
    assert!(s.account(2).unwrap().unwrap().off);
    s.put_account(2, "c@d", "2026-11-23").unwrap(); // a re-mint resets the windows
    assert!(s.account(2).unwrap().unwrap().wins.is_empty());
    assert!(s.del_account(2).unwrap() && !s.del_account(2).unwrap() && !s.del_account(LOGIN).unwrap());
    assert!(s.account(5).unwrap().is_none());
}

#[test]
fn pick_counts_live_load() {
    let s = temp();
    let none = HashMap::new();
    assert_eq!(s.pick_slot("opus", &[], 97.0, &none).unwrap(), Some(LOGIN));
    s.put_account(0, "a@b", "").unwrap();
    s.put_account(1, "c@d", "").unwrap();
    s.record_windows(0, &win(10.0), "opus").unwrap();
    s.record_windows(1, &win(12.0), "opus").unwrap();
    assert_eq!(s.pick_slot("opus", &[], 97.0, &none).unwrap(), Some(0));
    s.put_seat("cos", "cos", "fable", "high").unwrap();
    s.seat_slot("cos", 0).unwrap();
    s.seat_started("cos", "sid", 1).unwrap();
    assert_eq!(s.pick_slot("opus", &[], 97.0, &none).unwrap(), Some(1));
    let id = s.add_task(&t("x", None)).unwrap();
    assert_eq!(s.begin_run(id, 1, "opus", "high", &[]).unwrap(), Gate::Started { run: 1, slot: 1 });
    assert_eq!(s.begin_run(id, 1, "opus", "high", &[]).unwrap(), Gate::Started { run: 2, slot: 0 });
    let spread: HashMap<i64, f64> = [(1, 100.0)].into_iter().collect();
    assert_eq!(s.pick_slot("opus", &[], 97.0, &spread).unwrap(), Some(0));
}
