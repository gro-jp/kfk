mod common;
use common::*;

#[test]
fn seats_lifecycle() {
    let s = temp();
    assert!(s.seat("cos").unwrap().is_none());
    assert_eq!(s.free_orch(2).unwrap().as_deref(), Some("o1"));
    s.put_seat("cos", "cos", "fable", "high").unwrap();
    s.put_seat("o1", "orch", "opus", "high").unwrap();
    s.seat_slot("o1", 4).unwrap();
    s.seat_started("o1", "sid-1", 4242).unwrap();
    assert_eq!(s.free_orch(2).unwrap().as_deref(), Some("o2"));
    s.put_seat("o2", "orch", "opus", "high").unwrap();
    s.seat_started("o2", "sid-2", 4243).unwrap();
    assert_eq!(s.free_orch(2).unwrap(), None);
    s.seat_ctx("o1", 1234).unwrap();
    s.seat_restarts("o1", &[1.0, 2.0]).unwrap();
    s.seat_idle("o1", Some(5.0)).unwrap();
    let o = s.seat("o1").unwrap().unwrap();
    assert_eq!((o.role.as_str(), o.slot, o.sid.as_deref(), o.pid, o.ctx, o.state.as_str(), o.idle_since), ("orch", Some(4), Some("sid-1"), Some(4242), Some(1234), "up", Some(5.0)));
    assert_eq!((o.model.as_str(), o.effort.as_str(), o.restarts.len()), ("opus", "high", 2));
    assert!(o.since.is_some());
    s.seat_exited("o1", 1).unwrap(); // another pid: untouched
    assert_eq!(s.seat("o1").unwrap().unwrap().pid, Some(4242));
    s.seat_exited("o1", 4242).unwrap();
    assert_eq!(s.seat("o1").unwrap().unwrap().pid, None);
    s.seat_starting("o1").unwrap();
    let o = s.seat("o1").unwrap().unwrap();
    assert_eq!((o.state.as_str(), o.pid), ("up", None));
    assert!(kfk_util::now() - o.since.unwrap() < 5.0);
    s.seat_state("o2", "up").unwrap();
    assert_eq!(s.seat("o2").unwrap().unwrap().pid, Some(4243));
    s.seat_state("o2", "stopped").unwrap();
    assert_eq!(s.seat("o2").unwrap().unwrap().pid, None);
    assert_eq!(s.free_orch(2).unwrap().as_deref(), Some("o2"));
    s.put_seat("o2", "orch", "fable", "max").unwrap(); // re-used with new parameters
    assert_eq!(s.seat("o2").unwrap().unwrap().model, "fable");
    assert_eq!(s.seats().unwrap().iter().map(|x| x.seat.clone()).collect::<Vec<_>>(), ["cos", "o1", "o2"]);
}
