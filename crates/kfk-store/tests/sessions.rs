mod common;
use common::*;

#[test]
fn sessions_and_their_spend() {
    let s = temp();
    assert!(s.session("a").unwrap().is_none());
    s.session_open("a", "cos", "cos", 3, "fable", "high").unwrap();
    s.session_open("b", "o1", "orch", 4, "opus", "high").unwrap();
    let a = s.session("a").unwrap().unwrap();
    assert_eq!((a.seat.as_str(), a.role.as_str(), a.slot, a.model.as_str(), a.effort.as_str(), a.ended, a.bytes_done, a.path.as_str(), a.last.as_str()), ("cos", "cos", Some(3), "fable", "high", None, 0, "", ""));
    assert!(a.started > 0.0);
    s.session_progress("a", "/t/a.jsonl", 100, "{}", &[(10, [1, 2, 3, 4], 1), (11, [10, 0, 0, 0], 1)]).unwrap();
    s.session_progress("a", "/t/a.jsonl", 250, "{\"mid\":\"x\"}", &[(11, [1, 1, 1, 1], 2)]).unwrap();
    s.session_progress("b", "/t/b.jsonl", 9, "", &[(11, [100, 0, 0, 0], 5)]).unwrap();
    let a = s.session("a").unwrap().unwrap();
    assert_eq!((a.path.as_str(), a.bytes_done, a.last.as_str()), ("/t/a.jsonl", 250, "{\"mid\":\"x\"}"));
    let all = s.spend_since(0.0).unwrap();
    assert_eq!(all.iter().map(|x| (x.seat.as_str(), x.role.as_str(), x.in_tok, x.cw_tok, x.cr_tok, x.out_tok, x.turns)).collect::<Vec<_>>(), [("cos", "cos", 12, 3, 4, 5, 4), ("o1", "orch", 100, 0, 0, 0, 5)]);
    assert_eq!(all[0].weighted(), kfk_util::weighted(12, 3, 4, 5));
    let late = s.spend_since(11.0 * 3600.0 + 1800.0).unwrap(); // the hour that contains `since` counts whole
    assert_eq!(late[0].in_tok, 11);
    assert!(s.spend_since(12.0 * 3600.0).unwrap().is_empty());
    assert_eq!(s.spend_of("a").unwrap().turns, 4);
    assert_eq!(s.spend_of("nobody").unwrap().weighted(), 0);
    let hours = s.spend_by_hour("cos", 0.0).unwrap();
    assert_eq!(hours.iter().map(|(h, x)| (*h, x.in_tok, x.turns)).collect::<Vec<_>>(), [(10, 1, 1), (11, 11, 3)]);
    assert!(s.spend_by_hour("cos", 12.0 * 3600.0).unwrap().is_empty());
    s.event(Some(7), "note", "first");
    s.event(None, "rotate", "second");
    let ev = s.events_recent(10).unwrap();
    assert_eq!((ev[0].1, ev[0].3.as_str(), ev[0].4.as_str()), (None, "rotate", "second"));
    assert_eq!((ev[1].1, ev[1].3.as_str()), (Some(7), "note"));
    assert_eq!(s.events_recent(1).unwrap().len(), 1);
    // open and closed sessions; a resume re-opens and moves the slot
    assert_eq!(s.sessions(true).unwrap().len(), 2);
    s.session_end("a").unwrap();
    assert_eq!(s.sessions(true).unwrap().iter().map(|x| x.sid.clone()).collect::<Vec<_>>(), ["b"]);
    assert_eq!(s.sessions(false).unwrap().len(), 2);
    assert!(s.session("a").unwrap().unwrap().ended.is_some());
    s.session_open("a", "cos", "cos", 7, "fable", "high").unwrap();
    let a = s.session("a").unwrap().unwrap();
    assert_eq!((a.slot, a.ended, a.bytes_done), (Some(7), None, 250));
}

#[test]
fn a_seat_note_is_kept_until_the_node_starts_again() {
    let s = temp();
    s.put_seat("cos", "cos", "fable", "high").unwrap();
    assert_eq!(s.seat("cos").unwrap().unwrap().note, "");
    s.seat_note("cos", "trust").unwrap();
    assert_eq!(s.seat("cos").unwrap().unwrap().note, "trust");
    s.seat_started("cos", "sid", 1).unwrap();
    assert_eq!(s.seat("cos").unwrap().unwrap().note, "");
}

#[test]
fn an_older_database_gains_the_new_columns() {
    let d = std::env::temp_dir().join(format!("kfk-old-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    {
        let c = rusqlite::Connection::open(d.join("kfk.db")).unwrap();
        c.execute_batch("create table seats(seat text primary key, role text, slot integer, sid text, pid integer, since real, ctx integer, model text default '', effort text default '', state text default 'stopped', restarts text default '[]', idle_since real); insert into seats(seat,role) values('cos','cos');").unwrap();
    }
    let s = kfk_store::Store::open(d.to_str().unwrap()).unwrap();
    assert_eq!(s.seat("cos").unwrap().unwrap().note, "");
    drop(s);
    kfk_store::Store::open(d.to_str().unwrap()).unwrap(); // a second open: the column is there already
}
