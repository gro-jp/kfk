//! starting and stopping nodes
use crate::prelude::*;

pub fn start_node(s: &Store, seat: &str, resume: Option<&str>) -> Result<()> {
    let (sess, exe) = (s.cfg("tmux_session"), std::env::current_exe()?);
    let mut dir = s.cfg("project_dir");
    if dir.is_empty() {
        dir = std::env::current_dir()?.to_string_lossy().into_owned();
    }
    let q = |x: &str| format!("'{}'", x.replace('\'', "'\\''"));
    let mut cmd = format!("{} _node {}", q(&exe.to_string_lossy()), seat);
    if let Some(why) = resume {
        cmd += &format!(" --resume {}", q(why));
    }
    let mut env: Vec<String> = vec![];
    for k in ["PATH", "HOME", "LANG", "KFK_HOME", "KFK_CLAUDE", "KFK_TOKEN_FILE", "KFK_TMUX_SOCKET", "LLVM_PROFILE_FILE"] {
        if let Ok(v) = std::env::var(k) {
            env.extend(["-e".to_string(), format!("{k}={v}")]);
        }
    }
    let envr: Vec<&str> = env.iter().map(|x| x.as_str()).collect();
    let target = format!("={sess}:={seat}");
    let o = if !session_up(s) {
        tmux(&[&["new-session", "-d", "-s", &sess, "-n", seat, "-c", &dir, "-x", "220", "-y", "50"], &envr[..], &[&cmd]].concat())
    } else if window_up(s, seat) {
        tmux(&[&["respawn-window", "-k", "-t", &target, "-c", &dir], &envr[..], &[&cmd]].concat())
    } else {
        tmux(&[&["new-window", "-d", "-t", &format!("={sess}:"), "-n", seat, "-c", &dir], &envr[..], &[&cmd]].concat())
    };
    if !o.status.success() {
        bail!("tmux could not start {seat}: {}", String::from_utf8_lossy(&o.stderr).trim());
    }
    s.seat_starting(seat)?;
    Ok(())
}

pub fn stop_node(s: &Store, seat: &str, why: &str) -> Result<()> {
    if window_up(s, seat) {
        tmux(&["kill-window", "-t", &format!("={}:={}", s.cfg("tmux_session"), seat)]);
    }
    s.seat_state(seat, "stopped")?;
    s.event(None, "node-stop", &format!("{seat}: {why}"));
    Ok(())
}

/// start an orchestrator on demand (cos calls this through `kfk orch start`); returns its name
pub fn orch_start(s: &Store, vault: &dyn Vault, model: Option<&str>, effort: Option<&str>) -> Result<String> {
    let name = s.free_orch(s.cfgi("max_orchestrators"))?.ok_or_else(|| anyhow!("all {} orchestrators are up; route the stream to one of them (kfk status)", s.cfgi("max_orchestrators")))?;
    let (dm, de) = (s.cfg("orch_model"), s.cfg("orch_effort"));
    s.put_seat(&name, "orch", model.unwrap_or(&dm), effort.unwrap_or(&de))?;
    s.c.execute("update seats set sid=null, slot=null where seat=?1", [&name])?; // a fresh context and a fresh account choice
    ensure_slot(s, vault, &s.seat(&name)?.unwrap())?;
    start_node(s, &name, None)?;
    Ok(name)
}
