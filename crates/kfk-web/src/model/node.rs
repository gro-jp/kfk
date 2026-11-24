//! one node: what it is doing now, its queue, its sessions and its spend
use super::{card, node_row, Card, NodeRow};
use crate::probe::Probe;
use anyhow::{anyhow, Result};
use kfk_store::{Session, Spend, Store, TaskFilter};
use kfk_util::now;

pub struct NodeView {
    pub row: NodeRow,
    /// the last lines of its pane; empty when it is not up
    pub pane: String,
    /// whether the page may hold a live terminal; the reason when not
    pub term: Result<(), &'static str>,
    pub queue: Vec<Card>,
    pub sessions: Vec<(Session, Spend)>,
    /// weighted tokens for each of the last 24 hours, oldest first
    pub hours: Vec<(i64, i64)>,
}

/// the tail of a pane without its blank lines
pub fn pane_tail(pane: &str, n: usize) -> String {
    let l: Vec<&str> = pane.lines().map(str::trim_end).filter(|l| !l.is_empty()).collect();
    l[l.len().saturating_sub(n)..].join("\n")
}

/// one value per hour from `first` to `last`, zero where nothing was spent
pub fn fill_hours(spent: &[(i64, i64)], first: i64, last: i64) -> Vec<(i64, i64)> {
    (first..=last).map(|h| (h, spent.iter().find(|x| x.0 == h).map(|x| x.1).unwrap_or(0))).collect()
}

/// `local`: the server listens on a loopback address
pub fn node_view(s: &Store, p: &dyn Probe, local: bool, name: &str) -> Result<NodeView> {
    let seat = s.seat(name)?.ok_or_else(|| anyhow!("no node {name}"))?;
    let row = node_row(s, p, seat)?;
    let pane = if row.state == "up" { pane_tail(&p.pane(s, name), 40) } else { String::new() };
    let queue = s.tasks(&TaskFilter { owner: Some(name.into()), open: true, limit: 200, ..Default::default() })?.into_iter().map(|t| card(s, t)).collect::<Result<Vec<_>>>()?;
    let mut sessions = vec![];
    for se in s.sessions(false)?.into_iter().filter(|x| x.seat == name) {
        let spend = s.spend_of(&se.sid)?;
        sessions.push((se, spend));
    }
    let last = (now() / 3600.0) as i64;
    let spent: Vec<(i64, i64)> = s.spend_by_hour(name, (last - 23) as f64 * 3600.0)?.iter().map(|(h, x)| (*h, x.weighted())).collect();
    Ok(NodeView { row, pane, term: crate::shell::shell_usable(s, local), queue, sessions, hours: fill_hours(&spent, last - 23, last) })
}

pub fn nodes(s: &Store, p: &dyn Probe) -> Result<Vec<(NodeRow, Spend)>> {
    let mut v = vec![];
    for seat in s.seats()? {
        let total = s.spend_since(0.0)?.into_iter().find(|x| x.seat == seat.seat).unwrap_or_default();
        v.push((node_row(s, p, seat)?, total));
    }
    Ok(v)
}
