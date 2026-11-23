//! one line about the archive
use crate::prelude::*;

pub fn status(s: &Store) -> Result<String> {
    if !initialised(s) {
        return Ok("log archive: not set up (kfk logs init)".into());
    }
    if !running(s) {
        return Ok("log archive: PostgreSQL is not running (kfk logs sync starts it)".into());
    }
    let mut c = connect(s, "kfk")?;
    let r = c.query_one("select (select count(*) from log_files), (select count(*) from log_lines), pg_size_pretty(pg_database_size('kfk')), (select to_char(min(ts),'YYYY-MM-DD') from log_lines), (select extract(epoch from max(last_sync))::float8 from log_files)", &[])?;
    Ok(format!("log archive: {} files, {} lines, {} | oldest line {} | last sync {}", r.get::<_, i64>(0), r.get::<_, i64>(1), r.get::<_, String>(2), r.get::<_, Option<String>>(3).unwrap_or_else(|| "-".into()), r.get::<_, Option<f64>>(4).map(|t| kfk_util::stamp(t, "%F %H:%M")).unwrap_or_else(|| "-".into())))
}
