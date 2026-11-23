use kfk_logs::*;

#[test]
fn classify_lines() {
    let (ts, ty, j) = classify(br#"{"type":"assistant","timestamp":"2026-11-22T10:00:00.123Z","x":1}"#);
    assert_eq!((ts.as_deref(), ty.as_deref()), (Some("2026-11-22T10:00:00.123Z"), Some("assistant")));
    assert_eq!(j.unwrap()["x"], 1);
    let (ts, ty, j) = classify(br#"{"timestamp":"yesterday"}"#);
    assert!(ts.is_none() && ty.is_none() && j.is_some());
    assert_eq!(classify(b"plain text"), (None, None, None));
    assert_eq!(classify(b"[1,2]"), (None, None, None));
    assert_eq!(classify(br#"{"a":"x\u0000y"}"#), (None, None, None));
}
