//! the state store: one local SQLite file (WAL)
mod accounts;
mod claim;
mod config;
pub mod defaults;
pub mod model;
mod rows;
mod runs;
pub mod schema;
mod seats;
mod sessions;
mod tasks;

pub use defaults::DEFAULTS;
pub use model::*;

use anyhow::Result;
use rusqlite::{Connection, Row};

pub struct Store {
    pub c: Connection,
    pub kh: String,
}

impl Store {
    pub fn open(kh: &str) -> Result<Store> {
        std::fs::create_dir_all(kh)?;
        let c = Connection::open(format!("{kh}/kfk.db"))?;
        c.busy_timeout(std::time::Duration::from_secs(60))?;
        c.pragma_update(None, "journal_mode", "wal")?;
        c.pragma_update(None, "synchronous", "normal")?;
        c.execute_batch(schema::SCHEMA)?;
        for sql in schema::ADDED {
            let _ = c.execute(sql, []);
        }
        c.execute("insert or ignore into accounts(slot,label) values(?1,'saved-login')", [kfk_usage::LOGIN])?;
        Ok(Store { c, kh: kh.to_string() })
    }

    pub(crate) fn list<T>(&self, sql: &str, p: impl rusqlite::Params, f: fn(&Row) -> rusqlite::Result<T>) -> Result<Vec<T>> {
        let mut st = self.c.prepare(sql)?;
        let v = st.query_map(p, f)?.collect::<rusqlite::Result<Vec<T>>>()?;
        Ok(v)
    }

    /// a fresh store in a temporary directory (tests)
    pub fn temp() -> Store {
        let d = std::env::temp_dir().join(format!("kfk-{}", uuid::Uuid::new_v4().simple()));
        Store::open(d.to_str().unwrap()).unwrap()
    }
}
