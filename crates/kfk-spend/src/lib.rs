//! folding a session transcript into hourly token spend (input, cache write, cache read, output)
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub type Tok = [i64; 4];
const KEYS: [&str; 4] = ["input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens", "output_tokens"];

/// the last assistant message seen: a transcript repeats a message id once per content block
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct Last {
    pub mid: String,
    pub u: Tok,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Bucket {
    pub hour: i64,
    pub u: Tok,
    pub turns: i64,
}

/// (hour since the epoch, message id, usage) of an assistant line; None for anything else
pub fn usage_of(line: &str) -> Option<(i64, String, Tok)> {
    let v: Value = serde_json::from_str(line).ok()?;
    if v.get("type")?.as_str()? != "assistant" {
        return None;
    }
    let m = v.get("message")?;
    let u = m.get("usage")?;
    let ts = chrono::DateTime::parse_from_rfc3339(v.get("timestamp")?.as_str()?).ok()?.timestamp();
    Some((ts.div_euclid(3600), m.get("id")?.as_str()?.to_string(), KEYS.map(|k| u.get(k).and_then(Value::as_i64).unwrap_or(0))))
}

/// add the complete lines of `text` to hourly buckets; a repeated message id counts once, at its latest usage
pub fn fold(text: &str, last: &mut Last) -> Vec<Bucket> {
    let mut by: BTreeMap<i64, Bucket> = BTreeMap::new();
    for (hour, mid, u) in text.lines().filter_map(usage_of) {
        let b = by.entry(hour).or_insert_with(|| Bucket { hour, ..Default::default() });
        let same = mid == last.mid;
        for i in 0..4 {
            b.u[i] += u[i] - if same { last.u[i] } else { 0 };
        }
        b.turns += !same as i64;
        *last = Last { mid, u };
    }
    by.into_values().collect()
}
