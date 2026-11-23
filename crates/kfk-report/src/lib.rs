//! delivery and token numbers over a period: what passed, what it cost, and how much of it was coordination
use anyhow::Result;
use kfk_store::{Spend, Store, TaskFilter};
use std::collections::BTreeMap;

/// agent runs of one task class on one model/effort
#[derive(Default, Debug, Clone, PartialEq, serde::Serialize)]
pub struct Rung {
    pub class: String,
    pub model: String,
    pub runs: i64,
    pub pass: i64,
    pub fail: i64,
    pub limit: i64,
    pub weighted: i64,
    pub secs: i64,
}

impl Rung {
    pub fn pass_rate(&self) -> i64 {
        100 * self.pass / (self.pass + self.fail).max(1)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct StreamLine {
    pub id: i64,
    pub status: String,
    pub owner: String,
    pub done: i64,
    pub children: i64,
    pub weighted: i64,
    pub title: String,
}

#[derive(Default, Debug, Clone, serde::Serialize)]
pub struct Summary {
    pub since: f64,
    pub runs: i64,
    pub passed: i64,
    pub first_try: i64,
    pub agents: i64,
    pub nodes: i64,
    pub cost: f64,
    pub rungs: Vec<Rung>,
    pub spend: Vec<Spend>,
    pub streams: Vec<StreamLine>,
}

impl Summary {
    pub fn total(&self) -> i64 {
        self.agents + self.nodes
    }
    /// weighted tokens per passed task, counting the nodes; None when nothing passed
    pub fn per_pass(&self) -> Option<i64> {
        (self.passed > 0).then(|| self.total() / self.passed)
    }
    pub fn per_pass_agents(&self) -> Option<i64> {
        (self.passed > 0).then(|| self.agents / self.passed)
    }
    /// the share of all weighted tokens spent by cos and the orchestrators, in percent
    pub fn coordination(&self) -> i64 {
        100 * self.nodes / self.total().max(1)
    }
}

pub fn summarise(s: &Store, since: f64, now: f64) -> Result<Summary> {
    let runs = s.runs_since(since)?;
    let (mut agg, mut classes): (BTreeMap<(String, String), Rung>, BTreeMap<i64, String>) = (BTreeMap::new(), BTreeMap::new());
    for r in &runs {
        let class = classes.entry(r.task).or_insert_with(|| s.task(r.task).map(|t| t.class).unwrap_or_default()).clone();
        let model = format!("{}/{}", r.model, r.effort);
        let a = agg.entry((class.clone(), model.clone())).or_insert_with(|| Rung { class, model, ..Default::default() });
        a.runs += 1;
        a.weighted += r.weighted();
        a.secs += (r.ended.unwrap_or(now) - r.started).max(0.0) as i64;
        match r.status.as_str() {
            "pass" => a.pass += 1,
            "fail" => a.fail += 1,
            "limit" => a.limit += 1,
            _ => {}
        }
    }
    let spend = s.spend_since(since)?;
    let mut streams = vec![];
    for t in s.tasks(&TaskFilter { streams: true, limit: 500, ..Default::default() })?.into_iter().filter(|t| t.updated > since) {
        let kids = s.children(t.id)?;
        let mut w = 0;
        for k in &kids {
            w += s.runs_of(k.id)?.iter().map(|r| r.weighted()).sum::<i64>();
        }
        streams.push(StreamLine { id: t.id, status: t.status, owner: t.owner, done: kids.iter().filter(|k| k.status == "done").count() as i64, children: kids.len() as i64, weighted: w, title: t.title });
    }
    Ok(Summary {
        since,
        runs: runs.len() as i64,
        passed: agg.values().map(|a| a.pass).sum(),
        first_try: runs.iter().filter(|r| r.status == "pass" && r.attempt == 1).count() as i64,
        agents: agg.values().map(|a| a.weighted).sum(),
        nodes: spend.iter().map(|x| x.weighted()).sum(),
        cost: runs.iter().map(|r| r.cost).sum(),
        rungs: agg.into_values().collect(),
        spend,
        streams,
    })
}
