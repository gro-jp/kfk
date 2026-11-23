//! one agent attempt, and one node session: their files and their transcript
use crate::probe::Probe;
use crate::source::{log, Log};
use anyhow::{anyhow, Result};
use kfk_store::{Run, Session, Spend, Store, Task};

pub struct RunView {
    pub run: Run,
    pub task: Option<Task>,
    pub prompt: String,
    pub result: String,
    pub check: String,
    pub log: Log,
}

pub fn run_view(s: &Store, p: &dyn Probe, id: i64, page: Option<usize>, all: bool) -> Result<RunView> {
    let run = s.run(id).map_err(|_| anyhow!("no run {id}"))?;
    let read = |f: &str| std::fs::read_to_string(format!("{}/{f}", run.dir)).unwrap_or_default();
    Ok(RunView { task: s.task(run.task).ok(), prompt: read("prompt.md"), result: read("result.md"), check: read("check.log"), log: log(s, p, &format!("{}/stream.jsonl", run.dir), page, all), run })
}

pub struct SessionView {
    pub session: Session,
    pub spend: Spend,
    pub log: Log,
}

pub fn session_view(s: &Store, p: &dyn Probe, sid: &str, page: Option<usize>, all: bool) -> Result<SessionView> {
    let session = s.session(sid)?.ok_or_else(|| anyhow!("no session {sid}"))?;
    Ok(SessionView { spend: s.spend_of(sid)?, log: log(s, p, &session.path, page, all), session })
}
