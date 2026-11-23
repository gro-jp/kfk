//! the PostgreSQL cluster kfk keeps beside its store
use crate::prelude::*;

pub const SCHEMA: &str = "
create table if not exists log_files(id bigserial primary key, path text unique not null, session text, source text,
  bytes_done bigint not null default 0, first_seen timestamptz default now(), last_sync timestamptz);
create table if not exists log_lines(file_id bigint not null references log_files(id), off bigint not null,
  ts timestamptz, type text, data jsonb, raw text, primary key(file_id, off));
create index if not exists log_lines_ts on log_lines(ts);
create index if not exists log_lines_type on log_lines(type);
";

pub fn pg(s: &Store, tool: &str) -> Command {
    // an empty pg_bin means the tools are on PATH
    let bin = s.cfg("pg_bin");
    Command::new(if bin.is_empty() { tool.to_string() } else { format!("{bin}/{tool}") })
}

pub fn data(s: &Store) -> String {
    format!("{}/pg", s.kh)
}

pub fn initialised(s: &Store) -> bool {
    Path::new(&format!("{}/PG_VERSION", data(s))).exists()
}

pub fn running(s: &Store) -> bool {
    pg(s, "pg_ctl").args(["-D", &data(s), "status"]).output().map(|o| o.status.success()).unwrap_or(false)
}

/// create the cluster once (local socket only, no network listener), start it, create the database and tables
pub fn init(s: &Store) -> Result<String> {
    let mut did = vec![];
    if !initialised(s) {
        let o = pg(s, "initdb").args(["-D", &data(s), "-U", "kfk", "--auth=trust", "-E", "UTF8", "--locale=C"]).output().map_err(|e| anyhow!("cannot run initdb ({e}): put PostgreSQL's bin directory on PATH, or kfk config pg_bin <dir>"))?;
        if !o.status.success() {
            bail!("initdb failed: {}", String::from_utf8_lossy(&o.stderr).trim());
        }
        did.push("cluster created");
    }
    if start(s)? {
        did.push("server started");
    }
    let mut c = connect(s, "postgres")?;
    if c.query("select 1 from pg_database where datname='kfk'", &[])?.is_empty() {
        c.batch_execute("create database kfk")?;
        did.push("database created");
    }
    connect(s, "kfk")?.batch_execute(SCHEMA)?;
    Ok(format!("log archive ready at {} (socket {}, port {}){}", data(s), s.kh, s.cfg("pg_port"), if did.is_empty() { String::new() } else { format!(": {}", did.join(", ")) }))
}

/// start the server if it is not running; true when this call started it
pub fn start(s: &Store) -> Result<bool> {
    if running(s) {
        return Ok(false);
    }
    std::fs::create_dir_all(format!("{}/log", s.kh))?;
    let opts = format!("-p {} -c listen_addresses='' -c unix_socket_directories='{}'", s.cfg("pg_port"), s.kh);
    let o = pg(s, "pg_ctl").args(["-D", &data(s), "-l", &format!("{}/log/pg.log", s.kh), "-o", &opts, "-w", "start"]).output()?;
    if !o.status.success() {
        bail!("PostgreSQL did not start: see {}/log/pg.log", s.kh);
    }
    Ok(true)
}

pub fn stop(s: &Store) -> bool {
    running(s) && pg(s, "pg_ctl").args(["-D", &data(s), "-m", "fast", "-w", "stop"]).output().map(|o| o.status.success()).unwrap_or(false)
}

pub fn connect(s: &Store, db: &str) -> Result<Client> {
    Client::connect(&format!("host={} port={} user=kfk dbname={db}", s.kh, s.cfg("pg_port")), NoTls).map_err(|e| anyhow!("cannot reach the log archive (kfk logs init): {e}"))
}
