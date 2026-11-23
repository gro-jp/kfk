//! entry points kfk starts for itself
use crate::ctx::{home, Ctx};
use anyhow::Result;
use std::io::Read;

pub fn node(seat: String, resume: Option<String>) -> Result<i32> {
    let c = Ctx::open()?;
    kfk_mizan::node_main(c.s, c.v.as_ref(), &seat, resume.as_deref())?;
    Ok(0)
}

/// a node's status line: feeds its account's usage windows and its context size into the store, at no token cost
pub fn statusline(c: &Ctx) -> Result<i32> {
    let mut j = String::new();
    std::io::stdin().read_to_string(&mut j)?;
    let seat = std::env::var("KFK_SEAT").unwrap_or_default();
    let (wins, used, model, line) = kfk_mizan::statusline(&j, &seat);
    if let Some(slot) = std::env::var("KFK_SLOT").ok().and_then(|x| x.parse::<i64>().ok()) {
        let _ = c.s.record_windows(slot, &wins, &model);
    }
    if !seat.is_empty() {
        let _ = c.s.seat_ctx(&seat, used);
    }
    println!("{line}");
    Ok(0)
}

pub fn mizan_loop(c: &Ctx) -> Result<i32> {
    kfk_mizan::mizan_loop(&home(), c.v.as_ref())
}

pub fn runner(c: &Ctx, id: i64) -> Result<i32> {
    kfk_engine::run_task(&c.s, c.v.as_ref(), id)?;
    Ok(0)
}

pub fn exec_loop(c: &Ctx, id: i64, jobs: usize) -> Result<i32> {
    let kh = c.s.kh.clone();
    kfk_engine::exec_stream(&c.s, id, jobs, &mut |t| kfk_engine::spawn_detached(&kh, &["_runner".into(), t.to_string()], &format!("T{t}")))?;
    Ok(0)
}
