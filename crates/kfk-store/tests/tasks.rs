mod common;
use common::*;

#[test]
fn tasks_crud_and_filters() {
    let s = temp();
    let a = s.add_task(&t("stream", None)).unwrap();
    let b = s.add_task(&Task { deps: "1, x,7".into(), timeout_s: Some(5), ..t("child", Some(a)) }).unwrap();
    assert!(s.add_task(&Task { class: "odd".into(), ..t("x", None) }).is_err());
    assert!(s.add_task(&t("orphan", Some(99))).is_err());
    assert!(s.task(99).is_err());
    let tb = s.task(b).unwrap();
    assert_eq!((tb.parent, tb.status.as_str(), tb.dep_ids(), tb.timeout_s), (Some(a), "todo", vec![1, 7], Some(5)));
    assert_eq!(s.children(a).unwrap().len(), 1);
    assert_eq!(s.open_children(a).unwrap(), 1);
    assert!(s.busy("o1").unwrap() && !s.busy("o9").unwrap());
    assert_eq!(s.open_counts("o1").unwrap(), (0, 2, 0));
    s.set_field(b, "check", "false").unwrap();
    s.set_field(b, "timeout", "9").unwrap();
    s.set_field(b, "title", "renamed").unwrap();
    assert!(s.set_field(b, "status", "done").is_err());
    assert!(s.set_field(99, "title", "x").is_err());
    let tb = s.task(b).unwrap();
    assert_eq!((tb.chk.as_str(), tb.timeout_s, tb.title.as_str()), ("false", Some(9), "renamed"));
    let f = |f: TaskFilter| s.tasks(&f).unwrap().iter().map(|x| x.id).collect::<Vec<_>>();
    assert_eq!(f(TaskFilter::default()), vec![a, b]);
    assert_eq!(f(TaskFilter { streams: true, ..Default::default() }), vec![a]);
    assert_eq!(f(TaskFilter { parent: Some(a), open: true, limit: 5, ..Default::default() }), vec![b]);
    assert_eq!(f(TaskFilter { owner: Some("zz".into()), ..Default::default() }), Vec::<i64>::new());
    assert_eq!(f(TaskFilter { status: Some("done".into()), ..Default::default() }), Vec::<i64>::new());
    s.set_status(b, "failed", Some("why")).unwrap();
    assert_eq!(s.task(b).unwrap().result, "why");
    s.set_status(b, "todo", None).unwrap();
    assert_eq!((s.task(b).unwrap().status, s.task(b).unwrap().result), ("todo".to_string(), "why".to_string()));
    s.set_status(a, "cancelled", None).unwrap();
    assert_eq!(s.task(b).unwrap().status, "cancelled");
    assert_eq!(s.events(Some(b), &["failed", "todo"], 10).unwrap().len(), 2);
    assert_eq!(s.events(None, &["add"], 1).unwrap().len(), 1);
}
