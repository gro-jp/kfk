//! mizan as a background process
use crate::prelude::*;

pub fn mizan_pid(kh: &str) -> Option<i64> {
    std::fs::read_to_string(format!("{kh}/mizan.pid")).ok().and_then(|x| x.trim().parse::<i64>().ok()).filter(|p| alive(*p))
}

pub fn mizan_start(kh: &str) -> Result<String> {
    if let Some(p) = mizan_pid(kh) {
        return Ok(format!("mizan already running (pid {p})"));
    }
    let c = spawn_detached(kh, &["_mizan".into()], "mizan")?;
    std::fs::write(format!("{kh}/mizan.pid"), c.id().to_string())?;
    Ok(format!("mizan started (pid {}; log {kh}/log/mizan.log)", c.id()))
}

pub fn mizan_stop(kh: &str) -> String {
    match mizan_pid(kh) {
        Some(p) => {
            unsafe { libc::kill(p as i32, libc::SIGTERM) };
            "mizan stopped".into()
        }
        None => "mizan is not running".into(),
    }
}

pub fn mizan_loop(kh: &str, vault: &dyn Vault) -> ! {
    println!("{} mizan start pid {}", stamp(now(), "%F %T"), std::process::id());
    loop {
        let every = match Store::open(kh).and_then(|s| {
            for a in round(&s, vault)? {
                println!("{} {a}", stamp(now(), "%F %T"));
            }
            if let Err(e) = kfk_logs::sync_quiet(&s) {
                println!("{} log archive: {e}", stamp(now(), "%F %T"));
            }
            Ok(s.cfgi("mizan_every"))
        }) {
            Ok(n) => n.max(1) as u64,
            Err(e) => {
                println!("{} round failed: {e}", stamp(now(), "%F %T"));
                60
            }
        };
        std::thread::sleep(std::time::Duration::from_secs(every));
    }
}
