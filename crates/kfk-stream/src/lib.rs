//! what one `claude -p --output-format stream-json` run left behind
use kfk_usage::{Win, Wins};
use regex::Regex;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Default, Debug)]
pub struct Parsed {
    pub input: i64,
    pub cw: i64,
    pub cr: i64,
    pub out: i64,
    pub cost: f64,
    pub turns: i64,
    pub text: String,
    pub error: bool,
    pub result: bool,
    pub limited: bool,
    pub auth: bool,
    pub wins: Wins,
}

fn n(v: &Value, k: &str) -> i64 {
    v.get(k).and_then(Value::as_i64).unwrap_or(0)
}

pub fn wins_from_event(info: &Value) -> Wins {
    let one = |w: &Value| Win {
        util: w.get("utilization").and_then(Value::as_f64).map(|u| (u * 1000.0).round() / 10.0),
        reset: w.get("resetsAt").and_then(Value::as_f64),
        status: w.get("status").and_then(Value::as_str).map(String::from),
    };
    match info.get("unifiedWindows").and_then(Value::as_object) {
        Some(m) if !m.is_empty() => m.iter().map(|(k, w)| (k.clone(), one(w))).collect(),
        _ => [(info.get("rateLimitType").and_then(Value::as_str).unwrap_or("unknown").to_string(), one(info))].into_iter().collect(),
    }
}

pub fn parse(text: &str) -> Parsed {
    let mut o = Parsed::default();
    let mut seen: BTreeMap<String, Value> = BTreeMap::new();
    for line in text.lines() {
        let Ok(d) = serde_json::from_str::<Value>(line) else { continue };
        match d.get("type").and_then(Value::as_str) {
            Some("assistant") => {
                if let Some(u) = d.pointer("/message/usage") {
                    let id = d.pointer("/message/id").and_then(Value::as_str).map(String::from).unwrap_or_else(|| seen.len().to_string());
                    seen.insert(id, u.clone());
                }
            }
            Some("rate_limit_event") => {
                let info = d.get("rate_limit_info").cloned().unwrap_or(Value::Null);
                o.wins.extend(wins_from_event(&info));
                if info.get("status").and_then(Value::as_str) == Some("rejected") {
                    o.limited = true;
                }
            }
            Some("result") => {
                let u = d.get("usage").cloned().unwrap_or(Value::Null);
                o.result = true;
                o.turns = n(&d, "num_turns");
                o.cost = d.get("total_cost_usd").and_then(Value::as_f64).unwrap_or(0.0);
                o.text = d.get("result").and_then(Value::as_str).unwrap_or("").to_string();
                o.error = d.get("is_error").and_then(Value::as_bool).unwrap_or(false);
                (o.input, o.cw, o.cr, o.out) = (n(&u, "input_tokens"), n(&u, "cache_creation_input_tokens"), n(&u, "cache_read_input_tokens"), n(&u, "output_tokens"));
            }
            _ => {}
        }
    }
    if !o.result {
        for u in seen.values() {
            o.input += n(u, "input_tokens");
            o.cw += n(u, "cache_creation_input_tokens");
            o.cr += n(u, "cache_read_input_tokens");
            o.out += n(u, "output_tokens");
        }
        o.text = kfk_util::tail_lines(text, 30, 1500);
    }
    if o.error || !o.result {
        let auth = Regex::new(r"\b401\b|access token is invalid|authentication_error|Invalid bearer token").unwrap();
        let lim = Regex::new(r"(?i)usage limit|rate limit|limit reached|\b429\b").unwrap();
        o.auth = auth.is_match(&o.text);
        o.limited = o.limited || lim.is_match(&o.text);
    }
    o
}
