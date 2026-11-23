//! the account pool and its usage windows
use crate::{model::Account, rows, Store};
use anyhow::Result;
use kfk_usage::{self as usage, Cand, Wins};
use kfk_util::now;
use rusqlite::{params, OptionalExtension};
use std::collections::HashMap;

impl Store {
    pub fn pool(&self) -> Result<Vec<Account>> {
        self.list("select * from accounts where slot>=0 order by slot", [], rows::account)
    }
    pub fn account(&self, slot: i64) -> Result<Option<Account>> {
        Ok(self.c.query_row("select * from accounts where slot=?1", [slot], rows::account).optional()?)
    }
    pub fn put_account(&self, slot: i64, label: &str, minted: &str) -> Result<()> {
        self.c.execute(
            "insert into accounts(slot,label,minted) values(?1,?2,?3) on conflict(slot) do update set label=excluded.label, minted=excluded.minted, windows='{}'",
            params![slot, label, minted],
        )?;
        Ok(())
    }
    pub fn set_off(&self, slot: i64, off: bool) -> Result<bool> {
        Ok(self.c.execute("update accounts set off=?1 where slot=?2 and slot>=0", params![off as i64, slot])? == 1)
    }
    pub fn del_account(&self, slot: i64) -> Result<bool> {
        Ok(self.c.execute("delete from accounts where slot=?1 and slot>=0", [slot])? == 1)
    }
    pub fn record_windows(&self, slot: i64, wins: &Wins, model: &str) -> Result<()> {
        if wins.is_empty() {
            return Ok(());
        }
        self.c.execute_batch("begin immediate")?;
        let cur: Option<String> = self.c.query_row("select windows from accounts where slot=?1", [slot], |r| r.get(0)).optional()?;
        if let Some(cur) = cur {
            let mut w = usage::parse(&cur);
            usage::merge(&mut w, wins, model);
            self.c.execute("update accounts set windows=?1, asof=?2 where slot=?3", params![serde_json::to_string(&w)?, now(), slot])?;
        }
        self.c.execute_batch("commit")?;
        Ok(())
    }
    /// the account with the most headroom for `model`, counting live agents and seats as load
    pub fn pick_slot(&self, model: &str, exclude: &[i64], limit: f64, spread: &HashMap<i64, f64>) -> Result<Option<i64>> {
        let cands: Vec<Cand> = self.pool()?.into_iter().map(|a| Cand { slot: a.slot, off: a.off, wins: a.wins }).collect();
        let mut load = spread.clone();
        for s in self.list("select slot from runs where status='running'", [], |r| r.get::<_, i64>(0))? {
            *load.entry(s).or_default() += 3.0;
        }
        for s in self.list("select slot from seats where state='up' and slot is not null", [], |r| r.get::<_, i64>(0))? {
            *load.entry(s).or_default() += 5.0;
        }
        Ok(usage::pick(&cands, model, exclude, limit, &load, now()))
    }
}
