//! the rows of the store
use kfk_usage::Wins;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Account {
    pub slot: i64,
    pub label: String,
    pub minted: String,
    pub off: bool,
    pub wins: Wins,
    pub asof: Option<f64>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Seat {
    pub seat: String,
    pub role: String,
    pub slot: Option<i64>,
    pub sid: Option<String>,
    pub pid: Option<i64>,
    pub since: Option<f64>,
    pub ctx: Option<i64>,
    pub model: String,
    pub effort: String,
    pub state: String,
    pub restarts: Vec<f64>,
    pub idle_since: Option<f64>,
    /// what the node is waiting for that only the owner can give (a dialog in its pane); empty otherwise
    pub note: String,
}

/// one Claude session of a node; a seat has many over time
#[derive(Debug, Clone, serde::Serialize)]
pub struct Session {
    pub sid: String,
    pub seat: String,
    pub role: String,
    pub slot: Option<i64>,
    pub model: String,
    pub effort: String,
    pub started: f64,
    pub ended: Option<f64>,
    pub path: String,
    pub bytes_done: i64,
    pub last: String,
}

/// token spend of the nodes of one seat over a period
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct Spend {
    pub seat: String,
    pub role: String,
    pub in_tok: i64,
    pub cw_tok: i64,
    pub cr_tok: i64,
    pub out_tok: i64,
    pub turns: i64,
}

impl Spend {
    pub fn weighted(&self) -> i64 {
        kfk_util::weighted(self.in_tok, self.cw_tok, self.cr_tok, self.out_tok)
    }
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Task {
    pub id: i64,
    pub parent: Option<i64>,
    pub owner: String,
    pub title: String,
    pub spec: String,
    pub dod: String,
    pub chk: String,
    pub repo: String,
    pub base: String,
    pub class: String,
    pub model: String,
    pub effort: String,
    pub timeout_s: Option<i64>,
    pub max_attempts: Option<i64>,
    pub deps: String,
    pub status: String,
    pub attempts: i64,
    pub runner: Option<i64>,
    pub result: String,
    pub created: f64,
    pub updated: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Run {
    pub id: i64,
    pub task: i64,
    pub attempt: i64,
    pub slot: i64,
    pub model: String,
    pub effort: String,
    pub pid: Option<i64>,
    pub status: String,
    pub started: f64,
    pub ended: Option<f64>,
    pub dir: String,
    pub wd: String,
    pub check_rc: Option<i64>,
    pub in_tok: i64,
    pub cw_tok: i64,
    pub cr_tok: i64,
    pub out_tok: i64,
    pub cost: f64,
    pub turns: i64,
    pub note: String,
}

/// (time, task, actor, kind, text) of an event
pub type EventRow = (f64, Option<i64>, String, String, String);

#[derive(Default)]
pub struct TaskFilter {
    pub owner: Option<String>,
    pub parent: Option<i64>,
    pub status: Option<String>,
    pub open: bool,
    pub streams: bool,
    pub limit: i64,
}

impl Task {
    pub fn runnable(&self) -> bool {
        !(self.spec.trim().is_empty() || self.dod.trim().is_empty() || self.chk.trim().is_empty())
    }
    pub fn dep_ids(&self) -> Vec<i64> {
        self.deps.split(',').filter_map(|d| d.trim().parse().ok()).collect()
    }
}

impl Run {
    pub fn weighted(&self) -> i64 {
        kfk_util::weighted(self.in_tok, self.cw_tok, self.cr_tok, self.out_tok)
    }
}

#[derive(Debug, PartialEq)]
pub enum Gate {
    Started { run: i64, slot: i64 },
    Full,
    NoAccount,
}
