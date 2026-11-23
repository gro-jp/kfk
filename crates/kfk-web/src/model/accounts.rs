//! the account pool and who is on it
use anyhow::Result;
use kfk_store::{Account, Store};
use kfk_util::now;

pub struct Window {
    pub name: String,
    pub util: Option<f64>,
    pub reset: Option<f64>,
    pub full: bool,
}

pub struct AccountRow {
    pub account: Account,
    pub windows: Vec<Window>,
    pub seats: Vec<String>,
    pub agents: usize,
}

/// a window's name as people say it
pub fn window_name(key: &str) -> String {
    key.replace("seven_day", "week").replace("five_hour", "5 hours").replace('_', " ").replace('@', " · ")
}

pub fn accounts(s: &Store) -> Result<Vec<AccountRow>> {
    let (t, seats, running) = (now(), s.seats()?, s.running()?);
    Ok(s.pool()?
        .into_iter()
        .map(|a| AccountRow {
            windows: a.wins.iter().filter(|(_, w)| !w.reset.map(|r| r < t).unwrap_or(false)).map(|(k, w)| Window { name: window_name(k), util: w.util, reset: w.reset, full: w.status.as_deref() == Some("rejected") }).collect(),
            seats: seats.iter().filter(|x| x.state == "up" && x.slot == Some(a.slot)).map(|x| x.seat.clone()).collect(),
            agents: running.iter().filter(|r| r.slot == a.slot).count(),
            account: a,
        })
        .collect())
}
