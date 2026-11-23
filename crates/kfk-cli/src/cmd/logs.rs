//! the log archive
use crate::ctx::Ctx;
use anyhow::{bail, Result};

pub fn logs(c: &Ctx, action: &str, path: Option<&str>) -> Result<i32> {
    match action {
        "cat" => {
            let Some(path) = path else { bail!("logs cat <path>: the path as kfk archived it (a session .jsonl or a run's stream.jsonl)") };
            for l in kfk_logs::read_lines(&c.s, path)? {
                println!("{l}");
            }
        }
        "init" => println!("{}", kfk_logs::init(&c.s)?),
        "sync" => {
            let r = kfk_logs::sync(&c.s)?;
            println!("archived {} new lines ({} bytes) from {} of {} files", r.lines, r.bytes, r.changed, r.files);
        }
        "status" => println!("{}", kfk_logs::status(&c.s)?),
        "stop" => println!("{}", if kfk_logs::stop(&c.s) { "PostgreSQL stopped" } else { "PostgreSQL was not running" }),
        _ => bail!("logs: init | sync | status | stop | cat <path>"),
    }
    Ok(0)
}
