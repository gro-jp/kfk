mod common;
use common::*;

#[test]
fn env_is_scrubbed() {
    let _g = ENV.lock().unwrap();
    std::env::set_var("CLAUDE_CODE_OAUTH_TOKEN", "leak");
    std::env::set_var("KFK_SEAT", "cos");
    let e = clean_env(Some("tok"), &[("X", "1".into())]);
    std::env::remove_var("CLAUDE_CODE_OAUTH_TOKEN");
    std::env::remove_var("KFK_SEAT");
    assert_eq!(e.iter().filter(|(k, _)| k == "CLAUDE_CODE_OAUTH_TOKEN").map(|(_, v)| v.as_str()).collect::<Vec<_>>(), ["tok"]);
    assert!(!e.iter().any(|(k, _)| k == "KFK_SEAT") && e.iter().any(|(k, v)| k == "X" && v == "1"));
    assert!(!clean_env(None, &[]).iter().any(|(k, _)| k == "CLAUDE_CODE_OAUTH_TOKEN"));
}
