mod common;
use common::*;

#[test]
fn ping_reads_the_verdict() {
    let _g = ENV.lock().unwrap();
    let s = Store::temp();
    std::env::set_var("KFK_CLAUDE", fake(&s.kh, r#"if [ "$CLAUDE_CODE_OAUTH_TOKEN" = bad ]; then echo 'API Error: 401 OAuth access token is invalid'; exit 1; fi
[ -n "$CLAUDE_CODE_OAUTH_TOKEN" ] && [ -z "$CLAUDE_CONFIG_DIR" ] && exit 9
echo '{"type":"result","is_error":false,"result":"OK","usage":{}}'"#));
    assert_eq!(ping(Some("good")).0, 0);
    assert_eq!(ping(Some("bad")).0, 1);
    assert_eq!(ping(None).0, 0);
    std::env::remove_var("KFK_CLAUDE");
}
