//! config values and the event log
use crate::{defaults::DEFAULTS, Store};
use anyhow::{bail, Result};
use kfk_util::now;
use rusqlite::{params, OptionalExtension};

impl Store {
    pub fn cfg(&self, k: &str) -> String {
        let v: Option<String> = self.c.query_row("select v from config where k=?1", [k], |r| r.get(0)).optional().ok().flatten();
        v.unwrap_or_else(|| DEFAULTS.iter().find(|d| d.0 == k).map(|d| d.1).unwrap_or("").to_string())
    }
    pub fn cfgi(&self, k: &str) -> i64 {
        self.cfg(k).parse::<f64>().unwrap_or(0.0) as i64
    }
    pub fn set_cfg(&self, k: &str, v: &str) -> Result<()> {
        if !DEFAULTS.iter().any(|d| d.0 == k) {
            bail!("unknown config key {k:?}");
        }
        self.c.execute("insert or replace into config(k,v) values(?1,?2)", [k, v])?;
        Ok(())
    }
    pub fn event(&self, task: Option<i64>, kind: &str, data: &str) {
        let actor = std::env::var("KFK_SEAT").unwrap_or_else(|_| "owner".into());
        let data: String = data.chars().take(2000).collect();
        let _ = self.c.execute("insert into events(ts,task,actor,kind,data) values(?1,?2,?3,?4,?5)", params![now(), task, actor, kind, data]);
    }
    pub fn events(&self, task: Option<i64>, kinds: &[&str], limit: i64) -> Result<Vec<(f64, String, String, String)>> {
        let kinds = kinds.iter().map(|k| format!("'{}'", k.replace('\'', ""))).collect::<Vec<_>>().join(",");
        let sql = format!("select ts,actor,kind,data from (select * from events where (?1 is null or task=?1) and kind in ({kinds}) order by id desc limit ?2) order by id");
        self.list(&sql, params![task, limit], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
    }
}
