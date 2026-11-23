//! what happened: the event log and the numbers of a period
use anyhow::Result;
use kfk_report::Summary;
use kfk_store::Store;
use kfk_util::now;

pub struct History {
    pub days: f64,
    pub summary: Summary,
    /// (time, task, actor, kind, text), newest first
    pub events: Vec<kfk_store::EventRow>,
}

pub fn history(s: &Store, days: f64) -> Result<History> {
    let days = if days.is_finite() && days > 0.0 { days.min(3650.0) } else { 1.0 };
    let t = now();
    Ok(History { days, summary: kfk_report::summarise(s, t - 86400.0 * days, t)?, events: s.events_recent(300)? })
}
