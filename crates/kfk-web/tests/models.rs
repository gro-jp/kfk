mod common;
use common::*;
use kfk_web::lane::Lane;
use kfk_web::model::*;
use kfk_web::source::{lines, log};

#[test]
fn the_overview_sorts_work_into_lanes_and_lists_what_needs_the_owner() {
    let w = world();
    let o = overview(&w.s, &Fake::default()).unwrap();
    assert!(o.fleet_up && o.mizan);
    assert_eq!(o.nodes.iter().map(|n| (n.seat.seat.as_str(), n.state.as_str())).collect::<Vec<_>>(), [("cos", "up"), ("o1", "up"), ("o2", "stopped")]);
    let cos = &o.nodes[0];
    assert_eq!((cos.account.as_str(), cos.util, cos.window), ("1 · one@x", Some(100.0), 200_000));
    let o1 = &o.nodes[1];
    assert_eq!((o1.running, o1.todo, o1.blocked), (1, 5, 1));
    assert_eq!(o.agents.len(), 1);
    assert_eq!((o.agents[0].title.as_str(), o.agents[0].stream), ("a running child", Some(w.flight)));
    let lane_of = |title: &str| o.cards.iter().find(|c| c.task.title == title).unwrap().lane;
    assert_eq!([lane_of("stream in flight"), lane_of("stream planned"), lane_of("stream queued"), lane_of("stream stuck"), lane_of("stream that asks"), lane_of("stream done"), lane_of("stream dropped")],
               [Lane::InFlight, Lane::Planned, Lane::Queued, Lane::Stuck, Lane::NeedsYou, Lane::Done, Lane::Dropped]);
    let f = o.cards.iter().find(|c| c.task.id == w.flight).unwrap();
    assert_eq!((f.done, f.total, f.running), (0, 2, 1));
    assert_eq!(o.cards[0].task.title, "stream dropped"); // newest first
    assert_eq!(o.needs.iter().map(|n| (n.who.clone(), n.href.clone())).collect::<Vec<_>>(), [("cos".to_string(), "/node/cos".to_string()), (format!("T{}", w.ask), format!("/task/{}", w.ask))]);
    assert_eq!(o.needs[1].what, "needs owner: which region?");
    assert!(o.day.nodes > 0);
    assert_eq!(account_label(&w.s, None), "saved login");
    assert_eq!(account_label(&w.s, Some(9)), "9");
    assert_eq!((account_label_short(-1), account_label_short(4)), ("login".to_string(), "4".to_string()));
    // a stream that asks through its own status shows its title when it gave no reason
    w.s.set_status(w.ask, "blocked", Some("needs owner")).unwrap();
    w.s.c.execute("update tasks set result='' where id=?1", [w.ask]).unwrap();
    assert!(!overview(&w.s, &Fake::default()).unwrap().needs.is_empty());
    // a fleet that is down shows no node note
    let down = overview(&w.s, &Fake { up: false, ..Default::default() }).unwrap();
    assert!(!down.fleet_up && down.needs.iter().all(|n| n.who != "cos"));
}

#[test]
fn a_task_view_carries_children_runs_and_the_failing_output() {
    let w = world();
    let v = task_view(&w.s, w.failed_kid).unwrap();
    assert_eq!(v.parent.as_ref().unwrap().id, w.flight);
    assert_eq!((v.runs.len(), v.check_tail.as_str(), v.last_words.as_str()), (1, "check said no", "last words"));
    assert!(v.events.iter().any(|e| e.2 == "note" && e.3 == "a note from o1"));
    let st = task_view(&w.s, w.flight).unwrap();
    assert_eq!((st.kids.len(), st.runs.len(), st.check_tail.as_str()), (2, 0, ""));
    assert!(st.parent.is_none());
    assert!(task_view(&w.s, 9999).err().unwrap().to_string().starts_with("no task"));
}

#[test]
fn runs_and_sessions_read_their_transcripts_from_disk_or_the_archive() {
    let w = world();
    let p = Fake::default();
    let v = run_view(&w.s, &p, w.run, None, false).unwrap();
    assert_eq!((v.prompt.as_str(), v.result.as_str(), v.check.as_str()), ("the prompt", "last words", "check said no"));
    assert_eq!((v.log.origin.as_str(), v.log.paging.total, v.log.entries.len()), ("disk", 4, 4)); // system, text, tool, result
    assert_eq!(run_view(&w.s, &p, w.run, Some(1), true).unwrap().log.entries.len(), 5); // and the bookkeeping line
    assert!(run_view(&w.s, &p, 9999, None, false).err().unwrap().to_string().starts_with("no run"));
    let sv = session_view(&w.s, &p, "sid-cos", None, false).unwrap();
    assert_eq!((sv.log.entries.len(), sv.spend.turns), (1, 5));
    assert!(session_view(&w.s, &p, "nope", None, false).err().unwrap().to_string().starts_with("no session"));
    // the file is gone: the archive answers, or the page says why it is empty
    std::fs::remove_file(format!("{}/cos.jsonl", w.s.kh)).unwrap();
    let gone = session_view(&w.s, &p, "sid-cos", None, false).unwrap();
    assert!(gone.log.entries.is_empty() && gone.log.origin.contains("neither on disk nor in the log archive"));
    let kept = Fake { archive: Some(vec!["{\"type\":\"user\",\"message\":{\"content\":\"from the archive\"}}".into()]), ..Default::default() };
    let l = log(&w.s, &kept, "/gone.jsonl", None, false);
    assert_eq!((l.origin.as_str(), l.entries[0].body.as_str()), ("archive", "from the archive"));
    assert!(lines(&w.s, &p, "/gone.jsonl").is_err());
}

#[test]
fn a_node_view_shows_its_screen_queue_sessions_and_hours() {
    let w = world();
    let p = Fake::default();
    let v = node_view(&w.s, &p, "cos").unwrap();
    assert_eq!(v.pane, " ❯ working on T1\n  cos | Fable | ctx 10k");
    assert_eq!((v.sessions.len(), v.hours.len()), (1, 24));
    assert_eq!(v.hours.iter().filter(|h| h.1 > 0).count(), 2);
    assert_eq!(v.hours.last().unwrap().1, kfk_util::weighted(1000, 0, 0, 10));
    let o1 = node_view(&w.s, &p, "o1").unwrap();
    assert!(o1.queue.len() >= 5);
    let o2 = node_view(&w.s, &p, "o2").unwrap();
    assert_eq!((o2.pane.as_str(), o2.row.state.as_str()), ("", "stopped"));
    assert!(node_view(&w.s, &p, "o9").err().unwrap().to_string().starts_with("no node"));
    let all = nodes(&w.s, &p).unwrap();
    assert_eq!(all.iter().map(|(n, sp)| (n.seat.seat.as_str(), sp.turns)).collect::<Vec<_>>(), [("cos", 5), ("o1", 0), ("o2", 0)]);
    assert_eq!(pane_tail("a\n\nb\nc\n", 2), "b\nc");
    assert_eq!(fill_hours(&[(5, 9)], 4, 6), [(4, 0), (5, 9), (6, 0)]);
}

#[test]
fn accounts_show_live_windows_and_who_is_on_them() {
    let w = world();
    let a = accounts(&w.s).unwrap();
    assert_eq!(a.len(), 2);
    assert_eq!(a[0].windows.iter().map(|x| (x.name.as_str(), x.util, x.full)).collect::<Vec<_>>(), [("5 hours", None, true), ("week", Some(97.0), false)]); // the expired window is gone
    assert_eq!((a[0].seats.clone(), a[0].agents), (vec!["cos".to_string(), "o1".to_string()], 1));
    assert!(a[1].account.off && a[1].windows.is_empty() && a[1].seats.is_empty());
    assert_eq!(window_name("seven_day_opus@opus"), "week opus · opus");
}

#[test]
fn history_takes_a_sane_period() {
    let w = world();
    assert_eq!(history(&w.s, 7.0, None).unwrap().days, 7.0);
    assert_eq!(history(&w.s, -1.0, None).unwrap().days, 1.0);
    assert_eq!(history(&w.s, f64::NAN, None).unwrap().days, 1.0);
    assert_eq!(history(&w.s, 1e9, None).unwrap().days, 3650.0);
    let h = history(&w.s, 1.0, None).unwrap();
    assert!(h.events.len() > 10 && h.summary.runs == 2);
}

#[test]
fn history_shows_events_a_page_at_a_time_newest_first() {
    let w = world();
    let before = w.s.events_count().unwrap() as usize;
    for i in 0..(2 * EVENTS_PER_PAGE) { w.s.event(None, "note", &format!("n{i}")); }
    let total = before + 2 * EVENTS_PER_PAGE;
    // no page asked: the newest
    let h = history(&w.s, 1.0, None).unwrap();
    assert_eq!((h.paging.page, h.paging.total, h.events.len()), (1, total, EVENTS_PER_PAGE));
    assert_eq!(h.paging.pages, total.div_ceil(EVENTS_PER_PAGE));
    assert_eq!(h.events[0].4, format!("n{}", 2 * EVENTS_PER_PAGE - 1));
    // the second page carries on where the first stopped
    let h2 = history(&w.s, 1.0, Some(2)).unwrap();
    assert_eq!((h2.paging.page, h2.events[0].4.clone()), (2, format!("n{}", EVENTS_PER_PAGE - 1)));
    // the last page holds the oldest events; a page past the end is the last one
    let last = history(&w.s, 1.0, Some(9999)).unwrap();
    assert_eq!((last.paging.page, last.events.len()), (last.paging.pages, total - (last.paging.pages - 1) * EVENTS_PER_PAGE));
    assert_eq!(history(&w.s, 1.0, Some(0)).unwrap().paging.page, 1);
}
