//! one attempt: launch, account for it, judge it
use crate::prelude::*;

#[derive(Debug, PartialEq)]
pub enum Outcome {
    Pass,
    Fail,
    Limit,
}

pub fn attempt(s: &Store, vault: &dyn Vault, t: &Task, rid: i64, slot: i64, model: &str, effort: &str) -> Result<Outcome> {
    let rdir = format!("{}/runs/{}", s.kh, rid);
    fs::create_dir_all(&rdir)?;
    let wd = workdir(s, t)?;
    fs::write(format!("{rdir}/prompt.md"), prompt(s, t, rid, &wd))?;
    s.run_place(rid, &rdir, &wd)?;
    let timeout = t.timeout_s.unwrap_or_else(|| s.cfgi("timeout_s")).max(1) as u64;
    let tok = vault.get(slot);
    let env = clean_env(tok.as_deref(), &[("KFK_TASK", t.id.to_string()), ("KFK_SLOT", slot.to_string()), ("CLAUDE_CODE_AUTO_COMPACT_WINDOW", s.cfg("agent_compact"))]);
    let out = File::create(format!("{rdir}/stream.jsonl"))?;
    let mut p = Command::new(claude_bin())
        .args(["-p", "--model", model, "--effort", effort, "--output-format", "stream-json", "--verbose"])
        .args(s.cfg("agent_flags").split_whitespace())
        .current_dir(&wd)
        .env_clear()
        .envs(env)
        .stdin(File::open(format!("{rdir}/prompt.md"))?)
        .stdout(out.try_clone()?)
        .stderr(out)
        .process_group(0)
        .spawn()
        .map_err(|e| anyhow!("cannot start {}: {e}", claude_bin()))?;
    s.run_pid(rid, p.id() as i64)?;
    let mut note = String::new();
    if wait_timeout(&mut p, timeout).is_none() {
        note = format!("killed at its {timeout}s limit; ");
        kill_group(&mut p, 10);
    }
    let o = stream::parse(&fs::read_to_string(format!("{rdir}/stream.jsonl")).unwrap_or_default());
    fs::write(format!("{rdir}/result.md"), &o.text)?;
    s.record_windows(slot, &o.wins, model)?;
    s.run_usage(rid, &o)?;
    if o.auth || (o.limited && (o.error || !o.result)) {
        if !o.wins.values().any(|w| w.status.as_deref() == Some("rejected")) {
            // nothing said which window is full: keep this account out of the choice for a while
            let w: Wins = [("blocked".to_string(), Win { util: Some(100.0), status: Some("rejected".into()), reset: Some(now() + if o.auth { 3600.0 } else { 900.0 }) })].into_iter().collect();
            s.record_windows(slot, &w, model)?;
        }
        s.run_end(rid, "limit", None, &format!("{} on this account; moved to another", if o.auth { "auth rejected" } else { "usage limit" }))?;
        return Ok(Outcome::Limit);
    }
    let rc = check(s, t, &wd, &format!("{rdir}/check.log"))?;
    let verdict = if rc == 0 { "check passed".to_string() } else { format!("check exit {rc}: kfk task show {}", t.id) };
    s.run_end(rid, if rc == 0 { "pass" } else { "fail" }, Some(rc), &format!("{note}{verdict}"))?;
    Ok(if rc == 0 { Outcome::Pass } else { Outcome::Fail })
}
