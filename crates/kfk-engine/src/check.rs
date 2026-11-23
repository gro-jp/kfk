//! the task's check command decides
use crate::prelude::*;

/// run the task's check in `wd`: its exit status, output into `log`
pub fn check(s: &Store, t: &Task, wd: &str, log: &str) -> Result<i64> {
    let f = File::create(log)?;
    let mut p = Command::new("bash")
        .args(["-c", &t.chk])
        .current_dir(wd)
        .env_remove("CLAUDE_CODE_OAUTH_TOKEN")
        .env("KFK_TASK", t.id.to_string())
        .env("KFK_DIR", wd)
        .stdin(Stdio::null())
        .stdout(f.try_clone()?)
        .stderr(f)
        .process_group(0)
        .spawn()?;
    Ok(match wait_timeout(&mut p, s.cfgi("check_timeout_s").max(1) as u64) {
        Some(rc) => rc as i64,
        None => {
            kill_group(&mut p, 10);
            let _ = fs::OpenOptions::new().append(true).open(log).map(|mut f| std::io::Write::write_all(&mut f, b"\n[kfk] the check itself ran past its time limit\n"));
            124
        }
    })
}
