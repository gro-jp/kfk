mod common;
use common::*;

#[test]
fn config_defaults_and_overrides() {
    let s = temp();
    assert_eq!(s.cfgi("max_agents"), 8);
    s.set_cfg("max_agents", "3").unwrap();
    assert_eq!(s.cfg("max_agents"), "3");
    assert!(s.set_cfg("nope", "1").is_err());
    assert_eq!(s.cfg("nope"), "");
    let again = Store::open(&s.kh).unwrap();
    assert_eq!(again.cfgi("max_agents"), 3);
}
