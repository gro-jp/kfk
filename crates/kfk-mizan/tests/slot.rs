use kfk_mizan::*;
use kfk_store::Store;
use kfk_token::{FileVault, Vault};
use kfk_usage::{Win, Wins, LOGIN};

fn full(u: f64) -> Wins {
    [("seven_day".to_string(), Win { util: Some(u), reset: None, status: None })].into_iter().collect()
}

#[test]
fn a_seat_gets_the_account_with_headroom_and_keeps_one_it_can_use() {
    let s = Store::temp();
    let v = FileVault(format!("{}/tok.json", s.kh));
    s.put_seat("cos", "cos", "fable", "high").unwrap();
    let seat = || s.seat("cos").unwrap().unwrap();
    // an empty pool: the saved login
    assert!(!usable(&s, &v, None));
    assert!(usable(&s, &v, Some(LOGIN)));
    assert_eq!(ensure_slot(&s, &v, &seat()).unwrap(), LOGIN);
    assert_eq!(ensure_slot(&s, &v, &seat()).unwrap(), LOGIN); // kept
    // a pool: the login is no longer usable, the emptiest account is
    for slot in [1, 2] {
        s.put_account(slot, "a@b", "").unwrap();
        v.put(slot, "tok", "a@b").unwrap();
    }
    s.record_windows(1, &full(90.0), "fable").unwrap();
    s.record_windows(2, &full(10.0), "fable").unwrap();
    assert!(!usable(&s, &v, Some(LOGIN)));
    assert_eq!(ensure_slot(&s, &v, &seat()).unwrap(), 2);
    assert!(usable(&s, &v, Some(2)));
    // its account is switched off or loses its token: it moves
    s.set_off(2, true).unwrap();
    assert!(!usable(&s, &v, Some(2)));
    assert_eq!(ensure_slot(&s, &v, &seat()).unwrap(), 1);
    v.del(1);
    assert!(!usable(&s, &v, Some(1)));
    // every account past the node limit: still better than no node
    s.set_off(2, false).unwrap();
    s.record_windows(2, &full(99.0), "fable").unwrap();
    assert_eq!(ensure_slot(&s, &v, &seat()).unwrap(), 1);
    s.set_off(1, true).unwrap();
    s.set_off(2, true).unwrap();
    assert_eq!(ensure_slot(&s, &v, &seat()).unwrap(), LOGIN); // nothing live in the pool
}

#[test]
fn moving_a_stopped_seat_only_records_the_new_account() {
    let s = Store::temp();
    let v = FileVault(format!("{}/tok.json", s.kh));
    s.put_seat("o1", "orch", "opus", "high").unwrap();
    for slot in [1, 2] {
        s.put_account(slot, "a@b", "").unwrap();
        v.put(slot, "tok", "a@b").unwrap();
    }
    s.seat_slot("o1", 1).unwrap();
    assert_eq!(move_seat(&s, &v, "o1", None, true, "test").unwrap(), "o1: slot 1 -> 2");
    assert_eq!(move_seat(&s, &v, "o1", Some(1), true, "test").unwrap(), "o1: slot 2 -> 1");
    assert_eq!(move_seat(&s, &v, "o1", Some(LOGIN), false, "test").unwrap(), "o1: slot 1 -> login");
    assert!(move_seat(&s, &v, "o1", Some(7), true, "test").unwrap_err().to_string().contains("slot 7 has no token"));
    assert!(move_seat(&s, &v, "o9", None, true, "test").unwrap_err().to_string().contains("no seat o9"));
    s.seat_slot("o1", 1).unwrap();
    s.record_windows(2, &full(99.0), "opus").unwrap();
    assert!(move_seat(&s, &v, "o1", None, true, "test").unwrap_err().to_string().contains("no other account has headroom"));
    assert_eq!(s.events(None, &["swap"], 10).unwrap().len(), 3);
}
