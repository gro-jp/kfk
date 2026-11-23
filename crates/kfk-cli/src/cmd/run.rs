//! running tasks: one agent per attempt, a stream's children in parallel
use crate::ctx::Ctx;
use anyhow::{bail, Result};
use kfk_engine::spawn_detached;
use kfk_util::{alive, ktok, table};

fn outcome(c: &Ctx, id: i64) -> Result<i32> {
    let t = c.s.task(id)?;
    for r in c.s.runs_of(id)? {
        println!("{}", super::task_show::run_line(&r));
    }
    println!("T{id} {} | {}", t.status.to_uppercase(), if t.result.is_empty() { &t.title } else { &t.result });
    Ok(if t.status == "done" { 0 } else { 3 })
}

pub fn run(c: &Ctx, id: i64, detach: bool) -> Result<i32> {
    if !c.s.claim(id)? {
        bail!("T{id} is {}: not runnable (already claimed, or finished); kfk task show {id}", c.s.task(id)?.status);
    }
    let mut p = spawn_detached(&c.s.kh, &["_runner".into(), id.to_string()], &format!("T{id}"))?;
    if detach {
        println!("T{id} started; kfk wait {id}");
        return Ok(0);
    }
    p.wait()?;
    outcome(c, id)
}

pub fn wait(c: &Ctx, id: i64) -> Result<i32> {
    loop {
        let t = c.s.task(id)?;
        if t.status != "running" {
            return outcome(c, id);
        }
        if t.runner.map(|p| !alive(p)).unwrap_or(false) {
            c.s.reconcile(&alive)?;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

/// one line per child and one for the stream; exit 0 only when every child that was not cancelled is done
pub fn summary(c: &Ctx, sid: i64) -> Result<i32> {
    let kids = c.s.children(sid)?;
    let (mut rows, mut total) = (vec![], 0);
    let count = |st: &str| kids.iter().filter(|k| k.status == st).count();
    for k in &kids {
        let runs = c.s.runs_of(k.id)?;
        let w: i64 = runs.iter().map(|r| r.weighted()).sum();
        total += w;
        rows.push(vec![format!("T{}", k.id), k.status.clone(), k.attempts.to_string(), runs.last().map(|r| format!("{}/{}", r.model, r.effort)).unwrap_or_else(|| "-".into()), ktok(w),
                       (if k.result.is_empty() { &k.title } else { &k.result }).chars().take(90).collect()]);
    }
    println!("{}", table(&["id", "status", "tries", "last model", "weighted", "result"], &rows));
    let n = kids.len() - count("cancelled");
    let all = n > 0 && count("done") == n;
    println!("stream T{sid}: {}/{n} done, {} failed, {} blocked{} | {} weighted tokens{}", count("done"), count("failed"), count("blocked"),
             if count("todo") > 0 { format!(", {} waiting on a dependency that did not finish", count("todo")) } else { String::new() }, ktok(total),
             if all { format!(" | all children done: integrate, then kfk task done {sid}") } else { String::new() });
    Ok(if all { 0 } else { 3 })
}

pub fn exec(c: &Ctx, id: i64, jobs: Option<usize>, detach: bool) -> Result<i32> {
    c.s.task(id)?;
    let kids = c.s.children(id)?;
    if kids.is_empty() {
        bail!("T{id} has no child tasks to execute: kfk task add \"...\" --parent {id} ...");
    }
    let bad: Vec<String> = kids.iter().filter(|k| k.status == "todo" && !k.runnable()).map(|k| format!("T{}", k.id)).collect();
    if !bad.is_empty() {
        bail!("these children lack a spec, a definition of done or a check: {} (kfk task set <id> spec=... dod=... check='...')", bad.join(", "));
    }
    let jobs = jobs.unwrap_or(c.s.cfgi("max_agents") as usize).max(1);
    let mut p = spawn_detached(&c.s.kh, &["_exec".into(), id.to_string(), jobs.to_string()], &format!("exec-T{id}"))?;
    if detach {
        println!("stream T{id} executing ({} children); kfk task ls --parent {id}", kids.len());
        return Ok(0);
    }
    p.wait()?;
    summary(c, id)
}
