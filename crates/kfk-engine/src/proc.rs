//! child processes: waiting with a limit, stopping a group
use crate::prelude::*;

pub fn wait_timeout(p: &mut Child, secs: u64) -> Option<i32> {
    let end = Instant::now() + Duration::from_secs(secs);
    loop {
        if let Ok(Some(st)) = p.try_wait() {
            return Some(st.code().unwrap_or(-1));
        }
        if Instant::now() >= end {
            return None;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// ask the child's process group to stop; after `grace` seconds, make it
pub fn kill_group(p: &mut Child, grace: u64) {
    unsafe { libc::killpg(p.id() as i32, libc::SIGTERM) };
    if wait_timeout(p, grace).is_none() {
        unsafe { libc::killpg(p.id() as i32, libc::SIGKILL) };
        let _ = p.wait();
    }
}
