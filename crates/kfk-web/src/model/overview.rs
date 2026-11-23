//! the fleet at a glance
use crate::lane::{lane, Lane};
use crate::probe::Probe;
use anyhow::Result;
use kfk_report::Summary;
use kfk_store::{Run, Seat, Store, Task, TaskFilter};
use kfk_usage::{util, LOGIN};
use kfk_util::now;

pub struct NodeRow {
    pub seat: Seat,
    pub state: String,
    pub account: String,
    pub util: Option<f64>,
    pub reset: Option<f64>,
    /// the context size at which the node compacts
    pub window: i64,
    pub running: i64,
    pub todo: i64,
    pub blocked: i64,
}

pub struct AgentRow {
    pub run: Run,
    pub title: String,
    pub stream: Option<i64>,
}

/// a task with its lane and, for a stream, how far its children are
pub struct Card {
    pub task: Task,
    pub lane: Lane,
    pub done: usize,
    pub total: usize,
    pub running: usize,
}

pub struct Need {
    pub who: String,
    pub what: String,
    pub href: String,
}

pub struct Overview {
    pub fleet_up: bool,
    pub mizan: bool,
    pub project: String,
    pub home: String,
    pub nodes: Vec<NodeRow>,
    pub agents: Vec<AgentRow>,
    pub max_agents: i64,
    pub cards: Vec<Card>,
    pub needs: Vec<Need>,
    pub day: Summary,
    pub now: f64,
}

pub fn account_label(s: &Store, slot: Option<i64>) -> String {
    match slot {
        None | Some(LOGIN) => "saved login".into(),
        Some(n) => s.account(n).ok().flatten().map(|a| format!("{n} · {}", a.label)).unwrap_or_else(|| n.to_string()),
    }
}

/// a slot as a short word
pub fn account_label_short(slot: i64) -> String {
    if slot == LOGIN { "login".into() } else { slot.to_string() }
}

pub fn node_row(s: &Store, p: &dyn Probe, seat: Seat) -> Result<NodeRow> {
    let (u, reset) = s.account(seat.slot.unwrap_or(LOGIN))?.map(|a| util(&a.wins, &seat.model, now())).unwrap_or((None, None));
    let (running, todo, blocked) = s.open_counts(&seat.seat)?;
    Ok(NodeRow { state: p.node_state(s, &seat), account: account_label(s, seat.slot), util: u, reset, window: s.cfgi(if seat.role == "cos" { "cos_compact" } else { "orch_compact" }), running, todo, blocked, seat })
}

pub fn card(s: &Store, task: Task) -> Result<Card> {
    let kids = s.children(task.id)?;
    let live: Vec<&Task> = kids.iter().filter(|k| k.status != "cancelled").collect();
    Ok(Card { lane: lane(&task, &kids), done: live.iter().filter(|k| k.status == "done").count(), total: live.len(), running: live.iter().filter(|k| k.status == "running").count(), task })
}

pub fn overview(s: &Store, p: &dyn Probe) -> Result<Overview> {
    let t = now();
    let nodes = s.seats()?.into_iter().map(|seat| node_row(s, p, seat)).collect::<Result<Vec<_>>>()?;
    let mut agents = vec![];
    for run in s.running()? {
        let task = s.task(run.task).ok();
        agents.push(AgentRow { title: task.as_ref().map(|t| t.title.clone()).unwrap_or_default(), stream: task.and_then(|t| t.parent), run });
    }
    let cards = s.tasks(&TaskFilter { streams: true, limit: 500, ..Default::default() })?.into_iter().rev().map(|task| card(s, task)).collect::<Result<Vec<_>>>()?;
    let mut needs: Vec<Need> = nodes.iter().filter(|n| n.state == "up" && !n.seat.note.is_empty()).map(|n| Need { who: n.seat.seat.clone(), what: n.seat.note.clone(), href: format!("/node/{}", n.seat.seat) }).collect();
    for c in cards.iter().filter(|c| c.lane == Lane::NeedsYou) {
        let ask = std::iter::once(&c.task).chain(s.children(c.task.id)?.iter()).find(|k| k.status == "blocked").map(|k| k.result.clone()).unwrap_or_default();
        needs.push(Need { who: format!("T{}", c.task.id), what: if ask.is_empty() { c.task.title.clone() } else { ask }, href: format!("/task/{}", c.task.id) });
    }
    Ok(Overview { fleet_up: p.fleet_up(s), mizan: p.mizan(s), project: s.cfg("project_dir"), home: s.kh.clone(), nodes, agents, max_agents: s.cfgi("max_agents"), cards, needs, day: kfk_report::summarise(s, t - 86400.0, t)?, now: t })
}
