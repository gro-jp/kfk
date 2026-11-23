//! tasks: streams and their micro-tasks
use crate::{model::{Task, TaskFilter}, rows, Store};
use anyhow::{anyhow, bail, Result};
use kfk_util::now;
use rusqlite::{params, OptionalExtension};

impl Store {
    pub fn add_task(&self, t: &Task) -> Result<i64> {
        if !["mech", "impl", "hard", "deep"].contains(&t.class.as_str()) {
            bail!("class is one of mech, impl, hard, deep (got {:?})", t.class);
        }
        if let Some(p) = t.parent {
            self.task(p)?;
        }
        self.c.execute(
            "insert into tasks(parent,owner,title,spec,dod,chk,repo,base,class,model,effort,timeout_s,max_attempts,deps,created,updated) values(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?15)",
            params![t.parent, t.owner, t.title, t.spec, t.dod, t.chk, t.repo, t.base, t.class, t.model, t.effort, t.timeout_s, t.max_attempts, t.deps, now()],
        )?;
        let id = self.c.last_insert_rowid();
        self.event(Some(id), "add", &t.title);
        Ok(id)
    }
    pub fn task(&self, id: i64) -> Result<Task> {
        self.c.query_row("select * from tasks where id=?1", [id], rows::task).optional()?.ok_or_else(|| anyhow!("no task T{id}"))
    }
    pub fn children(&self, parent: i64) -> Result<Vec<Task>> {
        self.list("select * from tasks where parent=?1 order by id", [parent], rows::task)
    }
    pub fn tasks(&self, f: &TaskFilter) -> Result<Vec<Task>> {
        let mut q = String::from("select * from tasks where (?1 is null or owner=?1) and (?2 is null or parent=?2) and (?3 is null or status=?3)");
        if f.open {
            q += " and status in ('todo','running','blocked')";
        }
        if f.streams {
            q += " and parent is null";
        }
        q += " order by id desc limit ?4";
        let mut v = self.list(&q, params![f.owner, f.parent, f.status, if f.limit > 0 { f.limit } else { 60 }], rows::task)?;
        v.reverse();
        Ok(v)
    }
    pub fn set_field(&self, id: i64, key: &str, val: &str) -> Result<()> {
        let col = match key {
            "check" => "chk",
            "timeout" => "timeout_s",
            "title" | "spec" | "dod" | "repo" | "base" | "class" | "model" | "effort" | "max_attempts" | "owner" | "deps" => key,
            _ => bail!("cannot set {key:?} (one of: title spec dod check repo base class model effort timeout max_attempts owner deps)"),
        };
        self.task(id)?;
        self.c.execute(&format!("update tasks set {col}=?1, updated=?2 where id=?3"), params![val, now(), id])?;
        Ok(())
    }
    pub fn set_status(&self, id: i64, status: &str, result: Option<&str>) -> Result<()> {
        let t = self.task(id)?;
        self.c.execute("update tasks set status=?1, result=?2, runner=null, updated=?3 where id=?4", params![status, result.unwrap_or(&t.result), now(), id])?;
        if status == "todo" {
            self.c.execute("update tasks set attempts=0 where id=?1", [id])?;
        }
        if status == "cancelled" {
            self.c.execute("update tasks set status='cancelled', updated=?1 where parent=?2 and status in ('todo','blocked')", params![now(), id])?;
        }
        self.event(Some(id), status, result.unwrap_or(""));
        Ok(())
    }
    pub fn open_children(&self, id: i64) -> Result<i64> {
        Ok(self.c.query_row("select count(*) from tasks where parent=?1 and status in ('todo','running','blocked')", [id], |r| r.get(0))?)
    }
}
