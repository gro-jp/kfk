//! one task: attempts with escalation
use crate::prelude::*;

/// the runner of one task: attempts with escalation until its check passes or the attempts run out
pub fn run_task(s: &Store, vault: &dyn Vault, tid: i64) -> Result<()> {
    s.set_runner(tid, std::process::id() as i64)?;
    let mut bad: Vec<i64> = vec![];
    let mut dry: Option<f64> = None; // since when no account has had headroom
    loop {
        let t = s.task(tid)?;
        if t.status != "running" {
            return Ok(());
        }
        let attempt_no = t.attempts + 1;
        if attempt_no > t.max_attempts.unwrap_or_else(|| s.cfgi("max_attempts")) {
            return s.finish(tid, "failed", &format!("check still failing after {} attempts", t.attempts));
        }
        let (model, effort) = route::route(&s.cfg(&format!("ladder_{}", t.class)), attempt_no, &t.model, &t.effort);
        let (rid, slot) = loop {
            match s.begin_run(tid, attempt_no, &model, &effort, &bad)? {
                Gate::Started { run, slot } => {
                    dry = None;
                    break (run, slot);
                }
                Gate::Full => std::thread::sleep(Duration::from_millis(1000)),
                Gate::NoAccount => {
                    let reset = s.pool()?.iter().filter(|a| !a.off).filter_map(|a| kfk_usage::util(&a.wins, &model, now()).1).fold(None, |m: Option<f64>, r| Some(m.map_or(r, |m| m.min(r))));
                    let why = format!("no account has headroom for {model}; earliest reset {}", until(reset));
                    if now() - *dry.get_or_insert_with(now) >= s.cfgi("capacity_wait_s") as f64 {
                        return s.finish(tid, "blocked", &why);
                    }
                    s.waiting(tid, &format!("waiting: {why}"))?;
                    std::thread::sleep(Duration::from_millis(2000));
                }
            }
        };
        match attempt(s, vault, &t, rid, slot, &model, &effort) {
            Err(e) => {
                s.run_end(rid, "error", None, &e.to_string())?;
                return s.finish(tid, "blocked", &format!("kfk could not run it: {e}"));
            }
            Ok(Outcome::Limit) => {
                bad.push(slot);
                s.unattempt(tid)?;
            }
            Ok(Outcome::Pass) => {
                let br = if std::path::Path::new(&format!("{}/wt/T{tid}", s.kh)).is_dir() { format!("branch kfk/T{tid}, ") } else { String::new() };
                return s.finish(tid, "done", &format!("{br}check passed on attempt {attempt_no} ({model}/{effort}, slot {})", if slot == LOGIN { "login".into() } else { slot.to_string() }));
            }
            Ok(Outcome::Fail) => {}
        }
    }
}
