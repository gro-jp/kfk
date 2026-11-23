use kfk_report::*;
use kfk_store::{Gate, Store, Task};
use kfk_stream::Parsed;

fn task(s: &Store, title: &str, parent: Option<i64>, class: &str) -> i64 {
    s.add_task(&Task { title: title.into(), parent, spec: "s".into(), dod: "d".into(), chk: "true".into(), class: class.into(), owner: "o1".into(), ..Default::default() }).unwrap()
}

fn run(s: &Store, tid: i64, attempt: i64, model: &str, status: &str, out: i64) {
    let Gate::Started { run, .. } = s.begin_run(tid, attempt, model, "medium", &[]).unwrap() else { panic!() };
    s.run_usage(run, &Parsed { input: 100, out, cost: 0.5, ..Default::default() }).unwrap();
    s.run_end(run, status, None, "").unwrap();
}

#[test]
fn an_empty_period_has_no_rates() {
    let s = Store::temp();
    let r = summarise(&s, 0.0, 10.0).unwrap();
    assert_eq!((r.runs, r.passed, r.total(), r.per_pass(), r.per_pass_agents(), r.coordination()), (0, 0, 0, None, None, 0));
    assert!(r.rungs.is_empty() && r.spend.is_empty() && r.streams.is_empty());
    assert_eq!(Rung::default().pass_rate(), 0);
}

#[test]
fn runs_nodes_and_streams_add_up() {
    let s = Store::temp();
    let st = task(&s, "stream", None, "impl");
    let (a, b) = (task(&s, "a", Some(st), "mech"), task(&s, "b", Some(st), "impl"));
    run(&s, a, 1, "haiku", "pass", 10);
    run(&s, b, 1, "sonnet", "fail", 20);
    run(&s, b, 2, "sonnet", "limit", 0);
    run(&s, b, 2, "sonnet", "pass", 30);
    run(&s, 9999, 1, "opus", "error", 0); // a run whose task is gone still counts
    s.c.execute("update tasks set status='done' where id=?1", [a]).unwrap();
    let hour = (kfk_util::now() / 3600.0) as i64;
    s.session_open("sid", "cos", "cos", 1, "fable", "high").unwrap();
    s.session_progress("sid", "/p", 1, "", &[(hour, [1000, 0, 0, 0], 3)]).unwrap();
    let now = kfk_util::now();
    let r = summarise(&s, now - 3600.0, now).unwrap();
    assert_eq!((r.runs, r.passed, r.first_try), (5, 2, 1));
    let w = |out: i64| kfk_util::weighted(100, 0, 0, out);
    assert_eq!(r.agents, w(10) + w(20) + w(0) + w(30) + w(0));
    assert_eq!((r.nodes, r.total()), (1000, r.agents + 1000));
    assert_eq!(r.per_pass(), Some(r.total() / 2));
    assert_eq!(r.per_pass_agents(), Some(r.agents / 2));
    assert_eq!(r.coordination(), 100 * 1000 / r.total());
    assert!((r.cost - 2.5).abs() < 1e-9);
    let rung = r.rungs.iter().find(|x| x.class == "impl").unwrap();
    assert_eq!((rung.model.as_str(), rung.runs, rung.pass, rung.fail, rung.limit, rung.pass_rate()), ("sonnet/medium", 3, 1, 1, 1, 50));
    assert_eq!(r.rungs.len(), 3);
    assert_eq!(r.spend[0].seat, "cos");
    assert_eq!(r.streams, vec![StreamLine { id: st, status: "todo".into(), owner: "o1".into(), done: 1, children: 2, weighted: w(10) + w(20) + w(0) + w(30), title: "stream".into() }]);
    // a later period sees nothing
    let later = summarise(&s, now + 7200.0, now + 7300.0).unwrap();
    assert_eq!((later.runs, later.nodes, later.streams.len()), (0, 0, 0));
}
