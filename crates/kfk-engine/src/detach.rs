//! starting kfk itself in the background (the closure below runs in the forked child, where coverage cannot see)
use crate::prelude::*;

/// start this same binary in a session of its own, so it outlives whoever asked
pub fn spawn_detached(kh: &str, args: &[String], log: &str) -> Result<Child> {
    fs::create_dir_all(format!("{kh}/log"))?;
    let f = fs::OpenOptions::new().create(true).append(true).open(format!("{kh}/log/{log}.log"))?;
    let mut c = Command::new(std::env::current_exe()?);
    c.args(args).stdin(Stdio::null()).stdout(f.try_clone()?).stderr(f);
    unsafe {
        c.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }
    Ok(c.spawn()?)
}
