//! the fleet: cos, orchestrators on demand, mizan
use crate::cli::OrchCmd;
use crate::ctx::Ctx;
use anyhow::{bail, Result};
use kfk_mizan as mz;
use std::os::unix::process::CommandExt;

pub fn up(c: &Ctx, dir: Option<String>, fresh: bool) -> Result<i32> {
    let dir = match dir {
        Some(d) => std::fs::canonicalize(&d)?.to_string_lossy().into_owned(),
        None if c.s.cfg("project_dir").is_empty() => std::env::current_dir()?.to_string_lossy().into_owned(),
        None => c.s.cfg("project_dir"),
    };
    c.s.set_cfg("project_dir", &dir)?;
    c.s.put_seat("cos", "cos", &c.s.cfg("cos_model"), &c.s.cfg("cos_effort"))?;
    let cos = c.s.seat("cos")?.unwrap();
    mz::ensure_slot(&c.s, c.v.as_ref(), &cos)?;
    let st = if cos.state == "up" { mz::node_state(&c.s, &cos) } else { "stopped" };
    if st != "up" {
        // cos is the owner's conversation: it is resumed unless a new one is asked for
        mz::start_node(&c.s, "cos", if cos.sid.is_some() && !fresh { Some("kfk up") } else { None })?;
    }
    for o in c.s.seats()?.into_iter().filter(|s| s.role == "orch" && s.state == "up" && mz::node_state(&c.s, s) != "up") {
        mz::start_node(&c.s, &o.seat, o.sid.as_ref().map(|_| "kfk up"))?;
    }
    println!("{}", mz::mizan_start(&c.s.kh)?);
    match kfk_web::web_start(&c.s.kh, &c.s.cfg("web_addr")) {
        Ok(m) => println!("{m}"),
        Err(e) => println!("web: {e}"),
    }
    let wait = first_screen(c);
    super::seats::ls(c)?;
    if let Some(w) = wait {
        println!("\ncos: {}", w.note());
    }
    println!("\nproject dir: {dir}\ndata: {}\ntalk to cos: kfk attach   (orchestrators appear as cos starts them)", c.s.kh);
    Ok(0)
}

/// what cos shows once its first screen is drawn (up to 10 s): a dialog only the owner can answer, or nothing
fn first_screen(c: &Ctx) -> Option<mz::Wait> {
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let pane = mz::capture(&c.s, "cos");
        let wait = mz::pane_wait(&pane);
        if wait.is_some() || pane.contains("ctx ") {
            return wait;
        }
    }
    None
}

pub fn down(c: &Ctx, force: bool) -> Result<i32> {
    let n = c.s.running()?.len();
    if n > 0 && !force {
        bail!("{n} agent(s) are still running; wait for them (kfk status), or kfk down --force (agents finish on their own; only the nodes stop)");
    }
    println!("{}", mz::mizan_stop(&c.s.kh));
    for s in c.s.seats()? {
        if s.state == "up" {
            mz::stop_node(&c.s, &s.seat, "kfk down")?;
        }
    }
    println!("down: nodes stopped; tasks, accounts and history are untouched");
    Ok(0)
}

pub fn attach(c: &Ctx) -> Result<i32> {
    if !mz::session_up(&c.s) {
        bail!("the fleet is not up (kfk up)");
    }
    let sess = format!("={}", c.s.cfg("tmux_session"));
    let verb = if std::env::var("TMUX").is_ok() { "switch-client" } else { "attach-session" };
    let e = std::process::Command::new("tmux").args([verb, "-t", &format!("{sess}:=cos")]).exec();
    bail!("cannot run tmux: {e}")
}

pub fn orch(c: &Ctx, cmd: OrchCmd) -> Result<i32> {
    match cmd {
        OrchCmd::Start { model, effort } => {
            if !mz::session_up(&c.s) {
                bail!("the fleet is not up (kfk up)");
            }
            println!("{}", mz::orch_start(&c.s, c.v.as_ref(), model.as_deref(), effort.as_deref())?);
        }
        OrchCmd::Stop { name } => {
            match c.s.seat(&name)? {
                Some(s) if s.role == "orch" => {}
                _ => bail!("{name:?} is not an orchestrator (kfk orch ls)"),
            }
            if c.s.busy(&name)? {
                bail!("{name} still owns open tasks (kfk task ls --owner {name} --open)");
            }
            mz::stop_node(&c.s, &name, "kfk orch stop")?;
            println!("{name} stopped");
        }
        OrchCmd::Ls => return super::seats::ls(c),
    }
    Ok(0)
}

pub fn mizan(c: &Ctx, action: &str) -> Result<i32> {
    match action {
        "start" => println!("{}", mz::mizan_start(&c.s.kh)?),
        "stop" => println!("{}", mz::mizan_stop(&c.s.kh)),
        "once" => {
            let acts = mz::round(&c.s, c.v.as_ref())?;
            println!("{}", if acts.is_empty() { "nothing to do".to_string() } else { acts.join("\n") });
        }
        "status" => {
            println!("mizan: {}", mz::mizan_pid(&c.s.kh).map(|p| format!("running, pid {p}")).unwrap_or_else(|| "not running".into()));
            for (ts, _, kind, data) in c.s.events(None, &["rotate", "requeued", "node-start", "node-stop", "swap"], 10)? {
                println!("  {} {kind} {data}", kfk_util::stamp(ts, "%m-%d %H:%M"));
            }
        }
        _ => bail!("mizan: start | stop | status | once"),
    }
    Ok(0)
}
