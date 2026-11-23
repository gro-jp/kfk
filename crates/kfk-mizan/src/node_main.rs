//! the process inside a node's window
use crate::prelude::*;

/// runs inside the node's tmux window: claude as a child on the seat's account, then a visible exit line
pub fn node_main(s: Store, vault: &dyn Vault, name: &str, resume: Option<&str>) -> Result<()> {
    let seat = s.seat(name)?.ok_or_else(|| anyhow!("no seat {name}"))?;
    let slot = ensure_slot(&s, vault, &seat)?;
    let tok = vault.get(slot);
    let label = s.account(slot)?.map(|a| a.label).unwrap_or_default();
    let orchs = s.cfgi("max_orchestrators");
    let text = (if seat.role == "cos" { kfk_roles::COS } else { kfk_roles::ORCH }).replace("{seat}", name).replace("{max}", &orchs.to_string());
    std::fs::create_dir_all(format!("{}/nodes", s.kh))?;
    let exe = std::env::current_exe()?;
    let sf = format!("{}/nodes/{name}.settings.json", s.kh);
    std::fs::write(&sf, serde_json::json!({"statusLine": {"type": "command", "command": format!("'{}' _statusline", exe.to_string_lossy())}, "crossSessionInbound": "accept"}).to_string())?;
    let mut args: Vec<String> = ["-n", name, "--model", &seat.model, "--effort", &seat.effort, "--append-system-prompt", &text, "--settings", &sf].iter().map(|x| x.to_string()).collect();
    args.extend(s.cfg("node_flags").split_whitespace().map(String::from));
    let sid = match (resume, &seat.sid) {
        (Some(why), Some(sid)) => {
            args.extend(["--resume".to_string(), sid.clone(), format!("kfk: this node was restarted ({why}); your session is kept. Read `kfk status` and `kfk task ls --owner {name} --open`, then continue from the task states.")]);
            sid.clone()
        }
        _ => {
            let sid = uuid::Uuid::new_v4().to_string();
            args.extend(["--session-id".to_string(), sid.clone()]);
            sid
        }
    };
    let compact = s.cfg(&format!("{}_compact", if seat.role == "cos" { "cos" } else { "orch" }));
    let env = clean_env(tok.as_deref(), &[("KFK_SEAT", name.to_string()), ("KFK_SLOT", slot.to_string()), ("KFK_ROLE", seat.role.clone()), ("KFK_ACCOUNT", label), ("KFK_HOME", s.kh.clone()), ("CLAUDE_CODE_AUTO_COMPACT_WINDOW", compact)]);
    let mut p = Command::new(claude_bin()).args(&args).env_clear().envs(env).spawn().map_err(|e| anyhow!("cannot start {}: {e}", claude_bin()))?;
    s.seat_started(name, &sid, p.id() as i64)?;
    s.session_open(&sid, name, &seat.role, slot, &seat.model, &seat.effort)?;
    s.event(None, "node-start", &format!("{name} slot {slot} {}", if resume.is_some() { "resume" } else { "fresh" }));
    let kh = s.kh.clone();
    drop(s);
    let rc = p.wait()?;
    Store::open(&kh)?.seat_exited(name, p.id() as i64)?;
    println!("\n[kfk] node {name} exited ({rc}) at {}. mizan restarts it; by hand: kfk up", stamp(now(), "%H:%M:%S"));
    std::thread::sleep(std::time::Duration::from_secs(86400 * 365));
    Ok(())
}
