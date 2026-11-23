//! one stream: its children in dependency order, in parallel
use crate::prelude::*;

/// run every open child of a stream: in dependency order, `jobs` at a time, until nothing more can start.
/// `start` launches the runner of one claimed task.
pub fn exec_stream(s: &Store, sid: i64, jobs: usize, start: &mut dyn FnMut(i64) -> Result<Child>) -> Result<()> {
    let mut procs: Vec<(i64, Child)> = vec![];
    loop {
        procs.retain_mut(|(_, p)| !matches!(p.try_wait(), Ok(Some(_))));
        let kids = s.children(sid)?;
        let done = |id: i64| kids.iter().find(|k| k.id == id).map(|k| k.status == "done").unwrap_or_else(|| s.task(id).map(|t| t.status == "done").unwrap_or(false));
        let ready: Vec<&Task> = kids.iter().filter(|k| k.status == "todo" && !procs.iter().any(|p| p.0 == k.id) && k.dep_ids().into_iter().all(done)).collect();
        if procs.is_empty() && ready.is_empty() {
            break;
        }
        for k in ready.into_iter().take(jobs.saturating_sub(procs.len())) {
            if s.claim(k.id)? {
                procs.push((k.id, start(k.id)?));
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    s.event(Some(sid), "exec-settled", "");
    Ok(())
}
