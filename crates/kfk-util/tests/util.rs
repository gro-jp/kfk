use kfk_util::*;


#[test]
fn ktok_scales() {
    assert_eq!(ktok(999), "999");
    assert_eq!(ktok(1500), "1k");
    assert_eq!(ktok(2_500_000), "2.5M");
}

#[test]
fn ago_units() {
    assert_eq!(ago(None, 100.0), "-");
    assert_eq!(ago(Some(95.0), 100.0), "5s");
    assert_eq!(ago(Some(0.0), 120.0), "2m");
    assert_eq!(ago(Some(0.0), 7200.0), "2h");
    assert_eq!(ago(Some(0.0), 172800.0), "2d");
}

#[test]
fn slots() {
    assert_eq!(parse_slots("3").unwrap(), vec![3]);
    assert_eq!(parse_slots("3,12").unwrap(), vec![3, 12]);
    assert_eq!(parse_slots("10-12, 1").unwrap(), vec![10, 11, 12, 1]);
    assert!(parse_slots("x").is_err());
    assert!(parse_slots("5-2").is_err());
    assert!(parse_slots("100").is_err());
}

#[test]
fn alive_self_and_not() {
    assert!(alive(std::process::id() as i64));
    assert!(!alive(0));
    assert!(!alive(-5));
}

#[test]
fn weights() {
    assert_eq!(weighted(10, 100, 1000, 50), 485);
}

#[test]
fn table_aligns() {
    let t = table(&["a", "bb"], &[vec!["xxx".into(), "y".into()]]);
    assert_eq!(t, "a    bb\nxxx  y");
}

#[test]
fn tails() {
    assert_eq!(tail_lines("a\nb\nc", 2, 100), "b\nc");
    assert_eq!(tail_lines("abcdef", 5, 3), "def");
    assert_eq!(until(None), "-");
    assert!(now() > 1.0e9);
}

#[test]
fn a_time_out_of_range_is_a_dash() {
    assert_eq!(kfk_util::stamp(1e18, "%F"), "-");
    assert_eq!(kfk_util::until(None), "-");
}

#[test]
fn the_data_home_is_the_env_then_the_pointer_then_the_default() {
    use kfk_util::{home_pointer, resolve_home};
    assert_eq!(resolve_home(Some("/e"), Some("/p"), "/u"), "/e");
    assert_eq!(resolve_home(Some(" "), Some(" /p\n"), "/u"), "/p");
    assert_eq!(resolve_home(None, Some(""), "/u"), "/u/.kfk");
    assert_eq!(resolve_home(None, None, "/u"), "/u/.kfk");
    assert_eq!(home_pointer(Some("/x"), "/u"), "/x/kfk/home");
    assert_eq!(home_pointer(Some(""), "/u"), "/u/.config/kfk/home");
    assert_eq!(home_pointer(None, "/u"), "/u/.config/kfk/home");
}
