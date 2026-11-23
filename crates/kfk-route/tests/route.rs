use kfk_route::*;

const L: &str = "sonnet:medium opus:high fable:high";

#[test]
fn climbs_and_caps() {
    assert_eq!(route(L, 1, "", ""), ("sonnet".into(), "medium".into()));
    assert_eq!(route(L, 2, "", ""), ("opus".into(), "high".into()));
    assert_eq!(route(L, 9, "", ""), ("fable".into(), "high".into()));
    assert_eq!(route(L, 0, "", ""), ("sonnet".into(), "medium".into()));
}

#[test]
fn overrides_and_empty() {
    assert_eq!(route(L, 2, "haiku", ""), ("haiku".into(), "high".into()));
    assert_eq!(route(L, 1, "", "max"), ("sonnet".into(), "max".into()));
    assert_eq!(route("", 1, "", ""), ("sonnet".into(), "medium".into()));
}
