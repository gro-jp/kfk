mod common;
use common::*;
use kfk_web::shell::{shell_open, shell_usable, shell_view, Open};

fn open<'a>(seat: &'a str, origin: Option<&'a str>) -> Open<'a> {
    Open { local: true, seat, host: Some("localhost:7878"), origin, socket: None, id: "9" }
}

#[test]
fn a_terminal_is_usable_on_this_machine_until_it_is_switched_off() {
    let w = world();
    assert_eq!(shell_usable(&w.s, true), Ok(()));
    assert!(shell_usable(&w.s, false).unwrap_err().contains("network address"));
    w.s.set_cfg("web_shell", "off").unwrap();
    assert!(shell_usable(&w.s, true).unwrap_err().contains("turned off"));
    w.s.set_cfg("web_shell", "on").unwrap();
    assert_eq!(shell_usable(&w.s, true), Ok(()));
}

#[test]
fn the_view_of_a_terminal_page_knows_the_node_and_its_state() {
    let (w, p) = (world(), Fake::default());
    let v = shell_view(&w.s, &p, true, "o1").unwrap();
    assert_eq!((v.seat.as_str(), v.state.as_str(), v.usable), ("o1", "up", Ok(())));
    assert_eq!(shell_view(&w.s, &p, true, "o2").unwrap().state, "stopped");
    assert!(shell_view(&w.s, &p, false, "o1").unwrap().usable.is_err());
    assert!(shell_view(&w.s, &p, true, "nope").err().unwrap().to_string().starts_with("no node"));
}

#[test]
fn an_admitted_request_gets_the_tmux_command_for_its_node() {
    let (w, p) = (world(), Fake::default());
    let a = shell_open(&w.s, &p, &open("o1", Some("http://localhost:7878"))).unwrap();
    assert_eq!(a.join(" "), "new-session -t kfk -s kfk-web-9 ; set-option destroy-unattached on ; set-option mouse on ; set-option status off ; select-window -t :=o1 ; set-hook session-window-changed kill-session");
    let b = shell_open(&w.s, &p, &Open { socket: Some("sock"), ..open("cos", None) }).unwrap();
    assert_eq!(b[..2], ["-L", "sock"]);
}

#[test]
fn a_refused_request_gets_a_status_and_a_reason() {
    let (w, p) = (world(), Fake::default());
    let code = |o: &Open| shell_open(&w.s, &p, o).unwrap_err();
    assert_eq!(code(&open("cos", Some("https://evil.example"))).0, 403);
    assert_eq!(code(&Open { local: false, ..open("cos", None) }).0, 403);
    assert_eq!(code(&Open { host: None, ..open("cos", None) }).0, 403);
    for bad in ["o9", "", "o1;kill-server", "=cos"] {
        assert_eq!(code(&open(bad, None)).0, 404, "{bad}");
    }
    let (status, why) = code(&open("o2", None));
    assert!(status == 409 && why.contains("o2 is stopped"));
    w.s.set_cfg("web_shell", "off").unwrap();
    assert_eq!(code(&open("cos", None)), (403, "Terminals are turned off (kfk config web_shell on).".to_string()));
}
