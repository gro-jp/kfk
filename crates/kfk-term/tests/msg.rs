use kfk_term::{attach_args, resize, seat_ok, size};

#[test]
fn a_resize_is_a_small_json_object() {
    assert_eq!(resize(r#"{"cols":100,"rows":40}"#), Some((100, 40)));
    for junk in ["", "hello", "{}", r#"{"cols":100}"#, r#"{"cols":"a","rows":2}"#, r#"{"cols":-1,"rows":2}"#, "[1,2]"] {
        assert_eq!(resize(junk), None, "{junk}");
    }
}

#[test]
fn sizes_are_kept_sensible() {
    assert_eq!(size(None, None), (120, 32));
    assert_eq!(size(Some(1), Some(1)), (20, 5));
    assert_eq!(size(Some(99999), Some(99999)), (500, 300));
    assert_eq!(resize(r#"{"cols":100000,"rows":0}"#), Some((500, 5)));
}

#[test]
fn only_plain_node_names_reach_tmux() {
    assert!(seat_ok("cos") && seat_ok("o12"));
    for bad in ["", "o1;kill-server", "a b", "../x", "o1:0", "=cos", "abcdefghijklmnopq"] {
        assert!(!seat_ok(bad), "{bad}");
    }
}

#[test]
fn the_client_gets_its_own_session_in_the_fleets_group() {
    let a = attach_args(None, "kfk", "o1", "7");
    assert_eq!(a.join(" "), "new-session -t kfk -s kfk-web-7 ; set-option destroy-unattached on ; set-option mouse on ; set-option status off ; select-window -t :=o1 ; set-hook session-window-changed kill-session");
    let b = attach_args(Some("sock"), "fleet", "cos", "x");
    assert_eq!(b[..2], ["-L", "sock"]);
    assert!(b.join(" ").contains("new-session -t fleet -s fleet-web-x ;") && b.contains(&":=cos".to_string()));
}
