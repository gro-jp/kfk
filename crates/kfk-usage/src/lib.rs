//! usage windows per account, and the choice of an account for a request
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

pub const LOGIN: i64 = -1; // pseudo-slot: the login saved in ~/.claude (no token injected)
pub const GENERAL: [&str; 2] = ["seven_day", "five_hour"]; // these bind every model; any other window is model-scoped

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Win {
    pub util: Option<f64>,
    pub reset: Option<f64>,
    pub status: Option<String>,
}
pub type Wins = BTreeMap<String, Win>;

pub fn fam(model: &str) -> &str {
    ["fable", "opus", "sonnet", "haiku"].into_iter().find(|f| model.contains(f)).unwrap_or(model)
}

pub fn parse(json: &str) -> Wins {
    serde_json::from_str(json).unwrap_or_default()
}

/// fold freshly seen windows into the stored ones; a model-scoped window is keyed `<name>@<family>`
pub fn merge(cur: &mut Wins, new: &Wins, model: &str) {
    for (name, w) in new {
        let key = if GENERAL.contains(&name.as_str()) || name.contains('@') { name.clone() } else { format!("{name}@{}", fam(model)) };
        cur.insert(key, w.clone());
    }
}

/// (worst live utilisation in percent, its reset) for a request on `model`; None when nothing is known
pub fn util(w: &Wins, model: &str, now: f64) -> (Option<f64>, Option<f64>) {
    let f = fam(model);
    let mut worst: (Option<f64>, Option<f64>) = (None, None);
    for (k, x) in w {
        let scope = k.split_once('@').map(|p| p.1).unwrap_or("");
        if !scope.is_empty() && !f.is_empty() && scope != f {
            continue;
        }
        if x.reset.map(|r| r < now).unwrap_or(false) {
            continue;
        }
        let u = if x.status.as_deref() == Some("rejected") { Some(100.0) } else { x.util };
        if let Some(u) = u {
            if worst.0.map(|c| u > c).unwrap_or(true) {
                worst = (Some(u), x.reset);
            }
        }
    }
    worst
}

pub struct Cand {
    pub slot: i64,
    pub off: bool,
    pub wins: Wins,
}

/// The account with the most headroom for `model`: lowest utilisation, then least load.
/// `Some(LOGIN)` when the pool is empty; `None` when every account is at its limit.
pub fn pick(cands: &[Cand], model: &str, exclude: &[i64], limit: f64, load: &HashMap<i64, f64>, now: f64) -> Option<i64> {
    let live: Vec<&Cand> = cands.iter().filter(|c| !c.off).collect();
    if live.is_empty() {
        return Some(LOGIN);
    }
    let mut best: Option<(f64, i64)> = None;
    for c in live {
        if exclude.contains(&c.slot) {
            continue;
        }
        let u = util(&c.wins, model, now).0;
        if u.map(|u| u >= limit).unwrap_or(false) {
            continue;
        }
        let score = u.unwrap_or(30.0) + load.get(&c.slot).copied().unwrap_or(0.0);
        if best.map(|b| score < b.0).unwrap_or(true) {
            best = Some((score, c.slot));
        }
    }
    best.map(|b| b.1)
}

pub fn fmt(w: &Wins, now: f64) -> String {
    let parts: Vec<String> = w
        .iter()
        .filter(|(_, x)| !x.reset.map(|r| r < now).unwrap_or(false))
        .map(|(k, x)| {
            let u = if x.status.as_deref() == Some("rejected") { "FULL".to_string() } else { x.util.map(|u| format!("{}%", u as i64)).unwrap_or_else(|| "?".into()) };
            format!("{} {}", k.replace("seven_day", "wk").replace("five_hour", "5h"), u)
        })
        .collect();
    if parts.is_empty() { "-".into() } else { parts.join(", ") }
}
