use kfk_spend::*;

fn line(ts: &str, mid: &str, u: [i64; 4]) -> String {
    format!(r#"{{"type":"assistant","timestamp":"{ts}","message":{{"id":"{mid}","usage":{{"input_tokens":{},"cache_creation_input_tokens":{},"cache_read_input_tokens":{},"output_tokens":{}}}}}}}"#, u[0], u[1], u[2], u[3])
}

#[test]
fn only_assistant_lines_with_usage_count() {
    assert!(usage_of("not json").is_none());
    assert!(usage_of(r#"{"type":"user","timestamp":"2026-11-22T10:00:00Z"}"#).is_none());
    assert!(usage_of(r#"{"timestamp":"2026-11-22T10:00:00Z"}"#).is_none());
    assert!(usage_of(r#"{"type":"assistant","timestamp":"2026-11-22T10:00:00Z"}"#).is_none());
    assert!(usage_of(r#"{"type":"assistant","timestamp":"2026-11-22T10:00:00Z","message":{"id":"m"}}"#).is_none());
    assert!(usage_of(r#"{"type":"assistant","message":{"id":"m","usage":{}}}"#).is_none());
    assert!(usage_of(r#"{"type":"assistant","timestamp":"yesterday","message":{"id":"m","usage":{}}}"#).is_none());
    assert!(usage_of(r#"{"type":"assistant","timestamp":"2026-11-22T10:00:00Z","message":{"usage":{}}}"#).is_none());
    let (hour, mid, u) = usage_of(&line("1970-01-01T02:30:00.000Z", "m1", [1, 2, 3, 4])).unwrap();
    assert_eq!((hour, mid.as_str(), u), (2, "m1", [1, 2, 3, 4]));
    assert_eq!(usage_of(r#"{"type":"assistant","timestamp":"1970-01-01T00:00:00Z","message":{"id":"m","usage":{}}}"#).unwrap().2, [0; 4]);
}

#[test]
fn a_repeated_message_counts_once_at_its_latest_usage() {
    let text = [line("1970-01-01T01:00:00Z", "a", [10, 0, 100, 1]), line("1970-01-01T01:00:01Z", "a", [10, 0, 100, 7]), line("1970-01-01T01:59:00Z", "b", [5, 5, 5, 5]), "noise".into(), line("1970-01-01T02:00:00Z", "c", [1, 1, 1, 1])].join("\n");
    let mut last = Last::default();
    let b = fold(&text, &mut last);
    assert_eq!(b, vec![Bucket { hour: 1, u: [15, 5, 105, 12], turns: 2 }, Bucket { hour: 2, u: [1, 1, 1, 1], turns: 1 }]);
    assert_eq!(last, Last { mid: "c".into(), u: [1, 1, 1, 1] });
}

#[test]
fn the_cursor_carries_a_message_across_chunks() {
    let mut last = Last::default();
    assert_eq!(fold(&line("1970-01-01T01:00:00Z", "a", [10, 0, 0, 1]), &mut last)[0].u, [10, 0, 0, 1]);
    let b = fold(&line("1970-01-01T01:00:02Z", "a", [10, 0, 0, 9]), &mut last);
    assert_eq!(b, vec![Bucket { hour: 1, u: [0, 0, 0, 8], turns: 0 }]);
    assert!(fold("", &mut last).is_empty());
}
