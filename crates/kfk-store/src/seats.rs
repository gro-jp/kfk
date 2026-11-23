//! seats: the long-lived nodes (cos, orchestrators) and the account each is on
use crate::{model::Seat, rows, Store};
use anyhow::Result;
use kfk_util::now;
use rusqlite::{params, OptionalExtension};

impl Store {
    pub fn seats(&self) -> Result<Vec<Seat>> {
        self.list("select * from seats order by (seat!='cos'), length(seat), seat", [], rows::seat)
    }
    pub fn seat(&self, name: &str) -> Result<Option<Seat>> {
        Ok(self.c.query_row("select * from seats where seat=?1", [name], rows::seat).optional()?)
    }
    pub fn put_seat(&self, name: &str, role: &str, model: &str, effort: &str) -> Result<()> {
        self.c.execute(
            "insert into seats(seat,role,model,effort) values(?1,?2,?3,?4) on conflict(seat) do update set model=excluded.model, effort=excluded.effort",
            [name, role, model, effort],
        )?;
        Ok(())
    }
    /// the lowest free orchestrator name o1..oN: one never used, else one that is stopped
    pub fn free_orch(&self, cap: i64) -> Result<Option<String>> {
        let seats = self.seats()?;
        let names: Vec<String> = (1..=cap).map(|i| format!("o{i}")).collect();
        Ok(names.iter().find(|n| !seats.iter().any(|s| &s.seat == *n)).or_else(|| names.iter().find(|n| seats.iter().any(|s| &s.seat == *n && s.state != "up"))).cloned())
    }
    pub fn seat_slot(&self, name: &str, slot: i64) -> Result<()> {
        self.c.execute("update seats set slot=?1, since=?2 where seat=?3", params![slot, now(), name])?;
        Ok(())
    }
    /// the node's window was just asked to start: up, no pid yet, and the clock that gives it time to come up
    pub fn seat_starting(&self, name: &str) -> Result<()> {
        self.c.execute("update seats set state='up', pid=null, since=?1 where seat=?2", params![now(), name])?;
        Ok(())
    }
    pub fn seat_started(&self, name: &str, sid: &str, pid: i64) -> Result<()> {
        self.c.execute("update seats set sid=?1, pid=?2, since=?3, ctx=null, state='up', idle_since=null, note='' where seat=?4", params![sid, pid, now(), name])?;
        Ok(())
    }
    pub fn seat_exited(&self, name: &str, pid: i64) -> Result<()> {
        self.c.execute("update seats set pid=null where seat=?1 and pid=?2", params![name, pid])?;
        Ok(())
    }
    pub fn seat_state(&self, name: &str, state: &str) -> Result<()> {
        self.c.execute("update seats set state=?1, pid=case when ?1='up' then pid else null end where seat=?2", [state, name])?;
        Ok(())
    }
    pub fn seat_ctx(&self, name: &str, ctx: i64) -> Result<()> {
        self.c.execute("update seats set ctx=?1 where seat=?2", params![ctx, name])?;
        Ok(())
    }
    pub fn seat_restarts(&self, name: &str, r: &[f64]) -> Result<()> {
        self.c.execute("update seats set restarts=?1 where seat=?2", params![serde_json::to_string(r)?, name])?;
        Ok(())
    }
    pub fn seat_note(&self, name: &str, note: &str) -> Result<()> {
        self.c.execute("update seats set note=?1 where seat=?2", [note, name])?;
        Ok(())
    }
    pub fn seat_idle(&self, name: &str, since: Option<f64>) -> Result<()> {
        self.c.execute("update seats set idle_since=?1 where seat=?2", params![since, name])?;
        Ok(())
    }
    /// does this seat own an open task, or a task with a live agent?
    pub fn busy(&self, owner: &str) -> Result<bool> {
        let n: i64 = self.c.query_row("select count(*) from tasks where owner=?1 and status in ('todo','running','blocked')", [owner], |r| r.get(0))?;
        Ok(n > 0)
    }
    pub fn open_counts(&self, owner: &str) -> Result<(i64, i64, i64)> {
        let one = |st: &str| -> Result<i64> { Ok(self.c.query_row("select count(*) from tasks where owner=?1 and status=?2", [owner, st], |r| r.get(0))?) };
        Ok((one("running")?, one("todo")?, one("blocked")?))
    }
}
