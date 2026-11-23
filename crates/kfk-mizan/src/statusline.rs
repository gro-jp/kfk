//! what a node's status line feeds back
use crate::prelude::*;

/// what the status line of a node feeds back: (windows, context tokens, the line to show)
pub fn statusline(json: &str, seat: &str) -> (Wins, i64, String, String) {
    let d: Value = serde_json::from_str(json).unwrap_or(Value::Null);
    let model = d.pointer("/model/id").and_then(Value::as_str).unwrap_or("").to_string();
    let used: i64 = ["input_tokens", "cache_creation_input_tokens", "cache_read_input_tokens"].iter().map(|k| d.pointer(&format!("/context_window/current_usage/{k}")).and_then(Value::as_i64).unwrap_or(0)).sum();
    let mut wins = Wins::new();
    if let Some(m) = d.get("rate_limits").and_then(Value::as_object) {
        for (k, v) in m {
            wins.insert(k.clone(), Win { util: v.get("used_percentage").and_then(Value::as_f64), reset: v.get("resets_at").and_then(Value::as_f64), status: None });
        }
    }
    let wk = wins.get("seven_day").and_then(|w| w.util).map(|u| format!("{}%", u as i64)).unwrap_or_else(|| "?".into());
    let name = d.pointer("/model/display_name").and_then(Value::as_str).unwrap_or(&model);
    let line = format!("{} | {} | ctx {} | wk {}", if seat.is_empty() { "kfk" } else { seat }, name, ktok(used), wk);
    (wins, used, model, line)
}
