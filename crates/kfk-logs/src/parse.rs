//! lines of a transcript
use crate::prelude::*;

/// the complete lines of a buffer as (offset, bytes), and how many bytes they cover; a last line without its
/// newline is left for the next round
pub fn chunk(buf: &[u8]) -> (Vec<(usize, &[u8])>, usize) {
    let (mut out, mut start) = (vec![], 0);
    for (i, b) in buf.iter().enumerate() {
        if *b == b'\n' {
            if i > start {
                out.push((start, &buf[start..i]));
            }
            start = i + 1;
        }
    }
    (out, start)
}

/// (timestamp, type, json) of one line; a line that is not JSON, or that PostgreSQL's jsonb cannot hold, has no json
pub fn classify(line: &[u8]) -> (Option<String>, Option<String>, Option<Value>) {
    let text = String::from_utf8_lossy(line);
    if text.contains("\\u0000") {
        return (None, None, None);
    }
    match serde_json::from_str::<Value>(&text) {
        Ok(v) if v.is_object() => {
            let ts = v.get("timestamp").and_then(Value::as_str).filter(|t| t.len() >= 19 && t.as_bytes()[4] == b'-' && t.as_bytes()[10] == b'T').map(String::from);
            let ty = v.get("type").and_then(Value::as_str).map(String::from);
            (ts, ty, Some(v))
        }
        _ => (None, None, None),
    }
}
