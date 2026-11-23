//! the web interface as a service
use crate::ctx::Ctx;
use anyhow::{bail, Result};

pub fn web(c: &Ctx, action: &str) -> Result<i32> {
    let (kh, addr) = (c.s.kh.clone(), c.s.cfg("web_addr"));
    match action {
        "start" => println!("{}", kfk_web::web_start(&kh, &addr)?),
        "stop" => println!("{}", kfk_web::web_stop(&kh)),
        "status" => println!("web: {}", kfk_web::web_pid(&kh).map(|p| format!("running, pid {p}: http://{addr}")).unwrap_or_else(|| "not running (kfk web start)".into())),
        "run" => kfk_web::serve(&kh, &addr)?,
        _ => bail!("web: start | stop | status | run"),
    }
    Ok(0)
}
