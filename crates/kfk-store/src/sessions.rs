//! sessions: every Claude session a node has had, and its token spend by hour
use crate::{model::{Session, Spend}, rows, Store};
use anyhow::Result;
use kfk_util::now;
use rusqlite::{params, OptionalExtension};

impl Store {
    /// a node started or resumed this session; a resume on another account moves the session to that slot
    pub fn session_open(&self, sid: &str, seat: &str, role: &str, slot: i64, model: &str, effort: &str) -> Result<()> {
        self.c.execute(
            "insert into sessions(sid,seat,role,slot,model,effort,started) values(?1,?2,?3,?4,?5,?6,?7) on conflict(sid) do update set slot=excluded.slot, ended=null",
            params![sid, seat, role, slot, model, effort, now()],
        )?;
        Ok(())
    }
    pub fn session(&self, sid: &str) -> Result<Option<Session>> {
        Ok(self.c.query_row("select * from sessions where sid=?1", [sid], rows::session).optional()?)
    }
    pub fn sessions(&self, open_only: bool) -> Result<Vec<Session>> {
        self.list(if open_only { "select * from sessions where ended is null order by started" } else { "select * from sessions order by started desc" }, [], rows::session)
    }
    /// what was read of the transcript up to `bytes_done`: (hour, [input, cache write, cache read, output], turns)
    pub fn session_progress(&self, sid: &str, path: &str, bytes_done: i64, last: &str, buckets: &[(i64, [i64; 4], i64)]) -> Result<()> {
        self.c.execute_batch("begin immediate")?;
        let r = (|| -> Result<()> {
            for (hour, u, turns) in buckets {
                self.c.execute(
                    "insert into spend(sid,hour,in_tok,cw_tok,cr_tok,out_tok,turns) values(?1,?2,?3,?4,?5,?6,?7) on conflict(sid,hour) do update set in_tok=in_tok+excluded.in_tok, cw_tok=cw_tok+excluded.cw_tok, cr_tok=cr_tok+excluded.cr_tok, out_tok=out_tok+excluded.out_tok, turns=turns+excluded.turns",
                    params![sid, hour, u[0], u[1], u[2], u[3], turns],
                )?;
            }
            self.c.execute("update sessions set path=?1, bytes_done=?2, last=?3 where sid=?4", params![path, bytes_done, last, sid])?;
            Ok(())
        })();
        self.c.execute_batch(if r.is_ok() { "commit" } else { "rollback" })?;
        r
    }
    pub fn session_end(&self, sid: &str) -> Result<()> {
        self.c.execute("update sessions set ended=?1 where sid=?2 and ended is null", params![now(), sid])?;
        Ok(())
    }
    /// node spend per seat since `since` (whole hours)
    pub fn spend_since(&self, since: f64) -> Result<Vec<Spend>> {
        self.list(
            "select s.seat, s.role, sum(p.in_tok), sum(p.cw_tok), sum(p.cr_tok), sum(p.out_tok), sum(p.turns) from spend p join sessions s on s.sid=p.sid where p.hour>=?1 group by s.seat, s.role order by (s.seat!='cos'), length(s.seat), s.seat",
            [(since / 3600.0).floor() as i64],
            rows::spend,
        )
    }
    /// spend of one session, all hours
    pub fn spend_of(&self, sid: &str) -> Result<Spend> {
        Ok(self.c.query_row("select '', '', coalesce(sum(in_tok),0), coalesce(sum(cw_tok),0), coalesce(sum(cr_tok),0), coalesce(sum(out_tok),0), coalesce(sum(turns),0) from spend where sid=?1", [sid], rows::spend)?)
    }
    /// spend of one seat by hour since `since`, oldest first
    pub fn spend_by_hour(&self, seat: &str, since: f64) -> Result<Vec<(i64, Spend)>> {
        let mut st = self.c.prepare("select p.hour, s.seat, s.role, sum(p.in_tok), sum(p.cw_tok), sum(p.cr_tok), sum(p.out_tok), sum(p.turns) from spend p join sessions s on s.sid=p.sid where s.seat=?1 and p.hour>=?2 group by p.hour order by p.hour")?;
        let v = st.query_map(params![seat, (since / 3600.0).floor() as i64], |r| Ok((r.get(0)?, Spend { seat: r.get(1)?, role: r.get(2)?, in_tok: r.get(3)?, cw_tok: r.get(4)?, cr_tok: r.get(5)?, out_tok: r.get(6)?, turns: r.get(7)? })))?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(v)
    }
    /// every event, newest first
    /// newest first: `limit` events after skipping the `offset` newest
    pub fn events_page(&self, offset: i64, limit: i64) -> Result<Vec<crate::model::EventRow>> {
        self.list("select ts,task,actor,kind,data from events order by id desc limit ?2 offset ?1", [offset, limit], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))
    }
    pub fn events_count(&self) -> Result<i64> {
        Ok(self.c.query_row("select count(*) from events", [], |r| r.get(0))?)
    }
}
