//! the server process: bind, serve, and its life as a background service
use crate::live::Live;
use crate::routes::{router, App};
use anyhow::{bail, Result};
use kfk_util::alive;
use std::sync::Arc;

/// serve until killed
pub fn serve(kh: &str, addr: &str) -> Result<()> {
    let kh = kh.to_string();
    tokio::runtime::Builder::new_multi_thread().enable_all().build()?.block_on(async move {
        let l = tokio::net::TcpListener::bind(addr).await.map_err(|e| anyhow::anyhow!("cannot listen on {addr}: {e}"))?;
        let at = l.local_addr()?;
        let app = App { kh, probe: Arc::new(Live), local: at.ip().is_loopback() };
        println!("kfk web: http://{at}{}", if app.local { "" } else { " (a network address: terminals are read-only)" });
        axum::serve(l, router(app)).await?;
        Ok(())
    })
}

pub fn web_pid(kh: &str) -> Option<i64> {
    std::fs::read_to_string(format!("{kh}/web.pid")).ok().and_then(|x| x.trim().parse::<i64>().ok()).filter(|p| alive(*p))
}

pub fn web_start(kh: &str, addr: &str) -> Result<String> {
    if let Some(p) = web_pid(kh) {
        return Ok(format!("web already running (pid {p}): http://{addr}"));
    }
    let mut c = kfk_engine::spawn_detached(kh, &["_web".into()], "web")?;
    std::thread::sleep(std::time::Duration::from_millis(600));
    if let Ok(Some(_)) = c.try_wait() {
        bail!("the web interface did not start: {}", kfk_util::tail_lines(&std::fs::read_to_string(format!("{kh}/log/web.log")).unwrap_or_default(), 2, 400));
    }
    std::fs::write(format!("{kh}/web.pid"), c.id().to_string())?;
    Ok(format!("web started (pid {}): http://{addr}", c.id()))
}

pub fn web_stop(kh: &str) -> String {
    match web_pid(kh) {
        Some(p) => {
            unsafe { libc::kill(p as i32, libc::SIGTERM) };
            "web stopped".into()
        }
        None => "web is not running".into(),
    }
}
