mod common;
use common::*;
use std::process::Command;

#[test]
fn exec_orders_a_stream() {
    let s = Store::temp();
    let stream = task(&s, "stream", "x", "true", "");
    let kid = |title: &str, deps: &str| {
        s.add_task(&Task { title: title.into(), parent: Some(stream.id), spec: "s".into(), dod: "d".into(), chk: "true".into(), class: "impl".into(), deps: deps.into(), ..Default::default() }).unwrap()
    };
    let outside = task(&s, "a done task outside the stream", "x", "true", "");
    s.claim(outside.id).unwrap();
    s.finish(outside.id, "done", "").unwrap();
    let a = kid("a", "");
    let b = kid("b", &a.to_string());
    let c = kid("c", &outside.id.to_string());
    let d = kid("d fails", "");
    let e = kid("e waits on d for ever", &d.to_string());
    let f = kid("f waits on a task that does not exist", "9999");
    let mut order = vec![];
    let mut peak = 0usize;
    let other = Store::open(&s.kh).unwrap();
    exec_stream(&s, stream.id, 2, &mut |id| {
        order.push(id);
        peak = peak.max(other.tasks(&kfk_store::TaskFilter { status: Some("running".into()), limit: 50, ..Default::default() }).unwrap().len());
        other.finish(id, if id == d { "failed" } else { "done" }, "").unwrap(); // the runner's verdict
        Ok(Command::new("sleep").arg("0.3").spawn()?)
    })
    .unwrap();
    let st = |id: i64| s.task(id).unwrap().status;
    assert_eq!([st(a), st(b), st(c), st(d), st(e), st(f)], ["done", "done", "done", "failed", "todo", "todo"]);
    assert!(order.iter().position(|x| *x == a) < order.iter().position(|x| *x == b), "{order:?}");
    assert_eq!(order.len(), 4);
    assert!(peak <= 2, "never more than `jobs` at once: {peak}");
    assert_eq!(s.events(Some(stream.id), &["exec-settled"], 5).unwrap().len(), 1);
    // nothing open: settles at once; a failing start is an error
    exec_stream(&s, stream.id, 2, &mut |_| unreachable!()).unwrap();
    s.c.execute("update tasks set status='todo' where id=?1", [e]).unwrap();
    s.c.execute("update tasks set status='done' where id=?1", [d]).unwrap();
    assert!(exec_stream(&s, stream.id, 1, &mut |_| Err(anyhow::anyhow!("no runner"))).is_err());
    // a child someone else took between the listing and the claim is left to them
    let other_stream = task(&s, "stream 2", "x", "true", "");
    let mk = |title: &str| s.add_task(&Task { title: title.into(), parent: Some(other_stream.id), spec: "s".into(), dod: "d".into(), chk: "true".into(), class: "impl".into(), ..Default::default() }).unwrap();
    let (x, y) = (mk("x"), mk("y"));
    let mut started = vec![];
    exec_stream(&s, other_stream.id, 2, &mut |id| {
        started.push(id);
        other.c.execute("update tasks set status='done' where id=?1", [y]).unwrap();
        other.finish(id, "done", "").unwrap();
        Ok(Command::new("true").spawn()?)
    })
    .unwrap();
    assert_eq!(started, [x]);
}
