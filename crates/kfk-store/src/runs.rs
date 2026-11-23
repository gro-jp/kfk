//! runs: one row per agent attempt
use crate::{model::Run, rows, Store};
use anyhow::Result;
use kfk_util::now;
use rusqlite::{params, OptionalExtension};

impl Store {
    pub fn run(&self, id: i64) -> Result<Run> {
        Ok(self.c.query_row("select * from runs where id=?1", [id], rows::run)?)
    }
    pub fn runs_of(&self, task: i64) -> Result<Vec<Run>> {
        self.list("select * from runs where task=?1 order by id", [task], rows::run)
    }
    pub fn runs_since(&self, since: f64) -> Result<Vec<Run>> {
        self.list("select * from runs where started>?1 order by id", [since], rows::run)
    }
    pub fn running(&self) -> Result<Vec<Run>> {
        self.list("select * from runs where status='running' order by id", [], rows::run)
    }
    pub fn run_place(&self, id: i64, dir: &str, wd: &str) -> Result<()> {
        self.c.execute("update runs set dir=?1, wd=?2 where id=?3", params![dir, wd, id])?;
        Ok(())
    }
    pub fn run_pid(&self, id: i64, pid: i64) -> Result<()> {
        self.c.execute("update runs set pid=?1 where id=?2", params![pid, id])?;
        Ok(())
    }
    pub fn run_usage(&self, id: i64, p: &kfk_stream::Parsed) -> Result<()> {
        self.c.execute("update runs set in_tok=?1, cw_tok=?2, cr_tok=?3, out_tok=?4, cost=?5, turns=?6 where id=?7", params![p.input, p.cw, p.cr, p.out, p.cost, p.turns, id])?;
        Ok(())
    }
    pub fn run_end(&self, id: i64, status: &str, rc: Option<i64>, note: &str) -> Result<()> {
        self.c.execute("update runs set status=?1, check_rc=?2, ended=?3, note=?4 where id=?5", params![status, rc, now(), note, id])?;
        Ok(())
    }
    /// the directory of the newest earlier run of this task whose check failed
    pub fn last_failed_dir(&self, tid: i64, before: i64) -> Result<Option<String>> {
        Ok(self.c.query_row("select dir from runs where task=?1 and id<?2 and check_rc is not null and check_rc!=0 order by id desc limit 1", params![tid, before], |r| r.get(0)).optional()?)
    }
}
