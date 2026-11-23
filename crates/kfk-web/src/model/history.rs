//! what happened: the event log and the numbers of a period
use crate::paging::{paging, Paging};
use anyhow::Result;
use kfk_report::Summary;
use kfk_store::Store;
use kfk_util::now;

pub const EVENTS_PER_PAGE: usize = 50;

pub struct History {
    pub days: f64,
    pub summary: Summary,
    /// (time, task, actor, kind, text) of one page, newest first
    pub events: Vec<kfk_store::EventRow>,
    /// over every event ever recorded; page 1 is the newest
    pub paging: Paging,
}

/// `page` counts from 1; without it the newest events are shown
pub fn history(s: &Store, days: f64, page: Option<usize>) -> Result<History> {
    let days = if days.is_finite() && days > 0.0 { days.min(3650.0) } else { 1.0 };
    let t = now();
    let paging = paging(s.events_count()? as usize, EVENTS_PER_PAGE, Some(page.unwrap_or(1)));
    let events = s.events_page(paging.from as i64, EVENTS_PER_PAGE as i64)?;
    Ok(History { days, summary: kfk_report::summarise(s, t - 86400.0 * days, t)?, events, paging })
}
