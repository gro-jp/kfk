use kfk_stream::*;


    #[test]
    fn clean_result() {
        let s = r#"{"type":"system","subtype":"init"}
garbage line
{"type":"assistant","message":{"id":"m1","usage":{"input_tokens":1,"output_tokens":2}}}
{"type":"rate_limit_event","rate_limit_info":{"status":"allowed_warning","rateLimitType":"seven_day","utilization":0.96,"resetsAt":1795442400,"unifiedWindows":{"seven_day":{"utilization":0.96,"resetsAt":1795442400,"status":"allowed_warning"}}}}
{"type":"result","is_error":false,"num_turns":3,"total_cost_usd":0.5,"result":"done","usage":{"input_tokens":10,"cache_creation_input_tokens":100,"cache_read_input_tokens":1000,"output_tokens":50}}"#;
        let o = parse(s);
        assert!(o.result && !o.error && !o.limited && !o.auth);
        assert_eq!((o.input, o.cw, o.cr, o.out, o.turns), (10, 100, 1000, 50, 3));
        assert_eq!(o.text, "done");
        assert_eq!(o.wins["seven_day"].util, Some(96.0));
        assert_eq!(o.wins["seven_day"].reset, Some(1795442400.0));
    }

    #[test]
    fn killed_run_sums_distinct_messages() {
        let s = r#"{"type":"assistant","message":{"id":"m1","usage":{"input_tokens":1,"output_tokens":2}}}
{"type":"assistant","message":{"id":"m1","usage":{"input_tokens":1,"output_tokens":5}}}
{"type":"assistant","message":{"usage":{"input_tokens":3,"cache_read_input_tokens":7,"cache_creation_input_tokens":2,"output_tokens":1}}}"#;
        let o = parse(s);
        assert!(!o.result);
        assert_eq!((o.input, o.cw, o.cr, o.out), (4, 2, 7, 6));
    }

    #[test]
    fn limit_and_auth() {
        let o = parse(r#"{"type":"rate_limit_event","rate_limit_info":{"status":"rejected","rateLimitType":"five_hour","utilization":1.0}}
{"type":"result","is_error":true,"result":"Claude usage limit reached"}"#);
        assert!(o.limited && o.error && !o.auth);
        assert_eq!(o.wins["five_hour"].status.as_deref(), Some("rejected"));
        let a = parse(r#"{"type":"result","is_error":true,"result":"API Error: 401 OAuth access token is invalid"}"#);
        assert!(a.auth && !a.limited);
        let raw = parse("Invalid bearer token\n");
        assert!(raw.auth && !raw.result);
        assert!(parse("").text.is_empty());
        assert_eq!(wins_from_event(&serde_json::json!({})).keys().next().unwrap(), "unknown");
    }

#[test]
fn an_assistant_line_without_usage_is_skipped() {
    let o = kfk_stream::parse("{\"type\":\"assistant\",\"message\":{\"id\":\"m\"}}\n{\"type\":\"assistant\",\"message\":{\"usage\":{\"output_tokens\":3}}}");
    assert_eq!((o.out, o.result), (3, false));
}
