use kfk_term::guard::loopback_host;
use kfk_term::{admit, usable, Ask, Deny};

fn ask<'a>(host: Option<&'a str>, origin: Option<&'a str>) -> Ask<'a> {
    Ask { enabled: true, local: true, host, origin }
}

#[test]
fn only_names_of_this_machine_are_loopback() {
    for h in ["127.0.0.1:7878", "127.0.0.1", "localhost:7878", "LOCALHOST", "[::1]:7878", "[::1]", "127.9.9.9:1"] {
        assert!(loopback_host(h), "{h}");
    }
    for h in ["192.168.1.5:7878", "example.com", "example.com:7878", "localhost.evil.com:7878", "0.0.0.0:7878", "[2001:db8::1]:7878", "", "[", ":7878"] {
        assert!(!loopback_host(h), "{h}");
    }
}

#[test]
fn a_terminal_is_offered_only_when_switched_on_and_listening_locally() {
    assert_eq!(usable(true, true), Ok(()));
    assert_eq!(usable(false, true), Err(Deny::Off));
    assert_eq!(usable(true, false), Err(Deny::Network));
    assert_eq!(usable(false, false), Err(Deny::Off));
}

#[test]
fn kfks_own_page_on_this_machine_is_admitted() {
    assert_eq!(admit(&ask(Some("127.0.0.1:7878"), Some("http://127.0.0.1:7878"))), Ok(()));
    assert_eq!(admit(&ask(Some("localhost:7878"), Some("http://localhost:7878"))), Ok(()));
    assert_eq!(admit(&ask(Some("localhost:7878"), Some("https://localhost:7878"))), Ok(()));
    // a local program that is not a browser sends no origin
    assert_eq!(admit(&ask(Some("127.0.0.1:7878"), None)), Ok(()));
}

#[test]
fn another_sites_page_is_refused() {
    assert_eq!(admit(&ask(Some("127.0.0.1:7878"), Some("https://evil.example"))), Err(Deny::Origin));
    assert_eq!(admit(&ask(Some("127.0.0.1:7878"), Some("http://127.0.0.1:9999"))), Err(Deny::Origin));
    assert_eq!(admit(&ask(Some("127.0.0.1:7878"), Some("http://localhost:7878"))), Err(Deny::Origin));
    assert_eq!(admit(&ask(Some("127.0.0.1:7878"), Some("null"))), Err(Deny::Origin));
    assert_eq!(admit(&ask(Some("127.0.0.1:7878"), Some("ftp://127.0.0.1:7878"))), Err(Deny::Origin));
}

#[test]
fn a_name_that_is_not_this_machine_is_refused_even_from_its_own_page() {
    // DNS rebinding: evil.example resolves to 127.0.0.1, so the page and the socket share an origin
    assert_eq!(admit(&ask(Some("evil.example:7878"), Some("http://evil.example:7878"))), Err(Deny::Host));
    assert_eq!(admit(&ask(None, None)), Err(Deny::Host));
}

#[test]
fn the_switch_and_the_listening_address_come_before_everything() {
    let good = (Some("127.0.0.1:7878"), Some("http://127.0.0.1:7878"));
    assert_eq!(admit(&Ask { enabled: false, local: true, host: good.0, origin: good.1 }), Err(Deny::Off));
    assert_eq!(admit(&Ask { enabled: true, local: false, host: good.0, origin: good.1 }), Err(Deny::Network));
}

#[test]
fn every_refusal_says_why() {
    for d in [Deny::Off, Deny::Network, Deny::Host, Deny::Origin] {
        assert!(d.why().starts_with("Terminals "), "{d:?}");
    }
    assert!(Deny::Off.why().contains("kfk config web_shell on"));
}
