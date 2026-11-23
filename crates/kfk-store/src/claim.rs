//! taking a task for running, the concurrency gate, and recovery of orphans
use crate::{model::Gate, rows, Store};
use anyhow::{bail, Result};
use kfk_util::now;
use rusqlite::params;
use std::collections::HashMap;

impl Store {
    /// take a task for running: exactly one caller wins. Errors when it lacks a spec, a definition of done or a check.
    pub fn claim(&self, id: i64) -> Result<bool> {
        let t = self.task(id)?;
        if !t.runnable() {
            bail!("T{id} lacks a spec, a definition of done or a check command: an agent runs only against all three");
        }
        self.c.execute_batch("begin immediate")?;
        let n = self.c.execute("update tasks set status='running', runner=null, updated=?1, attempts=case when status='failed' then 0 else attempts end where id=?2 and status in ('todo','blocked','failed')", params![now(), id])?;
        self.c.execute_batch("commit")?;
        Ok(n == 1)
    }
    pub fn set_runner(&self, id: i64, pid: i64) -> Result<()> {
        self.c.execute("update tasks set runner=?1 where id=?2", params![pid, id])?;
        Ok(())
    }
    /// the concurrency gate and the account choice in one transaction
    pub fn begin_run(&self, tid: i64, attempt: i64, model: &str, effort: &str, exclude: &[i64]) -> Result<Gate> {
        self.c.execute_batch("begin immediate")?;
        let r = (|| -> Result<Gate> {
            let n: i64 = self.c.query_row("select count(*) from runs where status='running'", [], |r| r.get(0))?;
            if n >= self.cfgi("max_agents") {
                return Ok(Gate::Full);
            }
            let Some(slot) = self.pick_slot(model, exclude, self.cfgi("agent_max_util") as f64, &HashMap::new())? else { return Ok(Gate::NoAccount) };
            self.c.execute("insert into runs(task,attempt,slot,model,effort,status,started) values(?1,?2,?3,?4,?5,'running',?6)", params![tid, attempt, slot, model, effort, now()])?;
            let run = self.c.last_insert_rowid();
            self.c.execute("update tasks set attempts=?1, updated=?2 where id=?3", params![attempt, now(), tid])?;
            Ok(Gate::Started { run, slot })
        })();
        self.c.execute_batch(if r.is_ok() { "commit" } else { "rollback" })?;
        r
    }
    /// what a running task is waiting for, shown as its result until the runner's last word
    pub fn waiting(&self, tid: i64, why: &str) -> Result<()> {
        self.c.execute("update tasks set result=?1 where id=?2 and status='running' and result!=?1", params![why, tid])?;
        Ok(())
    }
    pub fn unattempt(&self, tid: i64) -> Result<()> {
        self.c.execute("update tasks set attempts=max(0,attempts-1) where id=?1", [tid])?;
        Ok(())
    }
    /// the runner's last word on a task; only while the task is still its own
    pub fn finish(&self, tid: i64, status: &str, result: &str) -> Result<()> {
        let n = self.c.execute("update tasks set status=?1, result=?2, runner=null, updated=?3 where id=?4 and status='running'", params![status, result, now(), tid])?;
        if n == 1 {
            self.event(Some(tid), status, result);
        }
        Ok(())
    }
    /// A task whose runner died goes back to todo and its run is marked lost. Returns (tasks requeued, agent pids
    /// that are still alive and must be stopped).
    pub fn reconcile(&self, alive: &dyn Fn(i64) -> bool) -> Result<(i64, Vec<i64>)> {
        let (mut n, mut kill) = (0, vec![]);
        for t in self.list("select * from tasks where status='running' and runner is not null", [], rows::task)? {
            if alive(t.runner.unwrap_or(0)) {
                continue;
            }
            for r in self.runs_of(t.id)?.into_iter().filter(|r| r.status == "running") {
                if let Some(p) = r.pid.filter(|p| alive(*p)) {
                    kill.push(p);
                }
                self.run_end(r.id, "lost", None, "its runner died")?;
            }
            self.c.execute("update tasks set status='todo', runner=null, attempts=max(0,attempts-1), updated=?1 where id=?2 and status='running'", params![now(), t.id])?;
            self.event(Some(t.id), "requeued", "runner died");
            n += 1;
        }
        Ok((n, kill))
    }
}
