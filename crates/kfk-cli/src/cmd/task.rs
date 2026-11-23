//! streams and micro-tasks
use crate::cli::{AddArgs, LsArgs, TaskCmd};
use crate::ctx::Ctx;
use anyhow::{bail, Result};
use kfk_store::{Task, TaskFilter};
use kfk_util::{ago, alive, now, table, tail_lines};
use std::io::Read;

/// text as given, `-` for stdin, `@file` for a file
fn text(v: Option<String>) -> Result<String> {
    Ok(match v.as_deref() {
        None => String::new(),
        Some("-") => { let mut s = String::new(); std::io::stdin().read_to_string(&mut s)?; s }
        Some(f) if f.starts_with('@') => std::fs::read_to_string(&f[1..])?,
        Some(t) => t.to_string(),
    })
}

fn add(c: &Ctx, a: AddArgs) -> Result<i32> {
    let parent = match a.parent { Some(p) => Some(c.s.task(p)?), None => None };
    let inherit = |own: Option<String>, f: fn(&Task) -> &String, fallback: String| own.or_else(|| parent.as_ref().map(|p| f(p).clone()).filter(|x| !x.is_empty())).unwrap_or(fallback);
    let repo = match a.repo { Some(r) => Some(std::fs::canonicalize(&r)?.to_string_lossy().into_owned()), None => None };
    let t = Task {
        parent: a.parent,
        owner: inherit(a.owner, |p| &p.owner, std::env::var("KFK_SEAT").unwrap_or_default()),
        repo: inherit(repo, |p| &p.repo, c.s.cfg("project_dir")),
        title: a.title, spec: text(a.spec)?, dod: text(a.dod)?, chk: a.check.unwrap_or_default(), base: a.base.unwrap_or_default(), class: a.class,
        model: a.model.unwrap_or_default(), effort: a.effort.unwrap_or_default(), timeout_s: a.timeout, max_attempts: a.max_attempts,
        deps: a.dep.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(","), ..Default::default()
    };
    let id = c.s.add_task(&t)?;
    println!("T{id}{}", if t.runnable() { "" } else { "  (it cannot run until it has a spec, a definition of done and a check)" });
    Ok(0)
}

pub fn ls(c: &Ctx, a: LsArgs) -> Result<i32> {
    let v = c.s.tasks(&TaskFilter { owner: a.owner, parent: a.parent, status: a.status, open: a.open, streams: a.streams, limit: a.limit })?;
    if a.json {
        println!("{}", serde_json::to_string(&v)?);
        return Ok(0);
    }
    let max = c.s.cfgi("max_attempts");
    let rows: Vec<Vec<String>> = v.iter().map(|t| vec![format!("T{}", t.id), t.parent.map(|p| format!("T{p}")).unwrap_or_else(|| "-".into()), t.status.clone(), if t.owner.is_empty() { "-".into() } else { t.owner.clone() },
        t.class.clone(), format!("{}/{}", t.attempts, t.max_attempts.unwrap_or(max)), ago(Some(t.updated), now()), t.title.chars().take(70).collect()]).collect();
    println!("{}", table(&["id", "stream", "status", "owner", "class", "tries", "age", "title"], &rows));
    Ok(0)
}

/// done only when the task's own check passes
fn done(c: &Ctx, id: i64, result: Option<String>, force: bool) -> Result<i32> {
    let t = c.s.task(id)?;
    if t.status == "running" && t.runner.map(alive).unwrap_or(false) {
        bail!("T{id} is being run by kfk right now (kfk wait {id})");
    }
    let open = c.s.open_children(id)?;
    if open > 0 && !force {
        bail!("T{id} still has {open} open child task(s): finish or cancel them (kfk task ls --parent {id} --open), or --force");
    }
    if !t.chk.trim().is_empty() {
        let wd = if !t.repo.is_empty() && std::path::Path::new(&t.repo).is_dir() { t.repo.clone() } else { ".".into() };
        let log = format!("{}/done-T{id}.log", c.s.kh);
        let rc = kfk_engine::check(&c.s, &t, &wd, &log)?;
        if rc != 0 {
            c.s.event(Some(id), "done-refused", &format!("check exit {rc}"));
            eprintln!("kfk: T{id} is NOT done: its check exited {rc} in {wd}\n{}", tail_lines(&std::fs::read_to_string(&log).unwrap_or_default(), 15, 2000));
            return Ok(3);
        }
    }
    c.s.set_status(id, "done", result.as_deref())?;
    println!("T{id} done{}", if t.chk.trim().is_empty() { " (no check on this task)" } else { " (check passed)" });
    Ok(0)
}

pub fn task(c: &Ctx, cmd: Option<TaskCmd>) -> Result<i32> {
    let mark = |id: i64, st: &str, result: Option<String>| -> Result<i32> {
        c.s.set_status(id, st, result.as_deref())?;
        println!("T{id} {st}");
        Ok(0)
    };
    match cmd {
        None => ls(c, LsArgs { open: true, limit: 60, ..Default::default() }),
        Some(TaskCmd::Add(a)) => add(c, *a),
        Some(TaskCmd::Ls(a)) => ls(c, a),
        Some(TaskCmd::Show { id, json }) => super::task_show::show(c, id, json),
        Some(TaskCmd::Set { id, kv }) => {
            for x in &kv {
                let Some((k, v)) = x.split_once('=') else { bail!("{x:?} is not key=value") };
                c.s.set_field(id, k, &text(Some(v.to_string()))?)?;
            }
            println!("T{id} updated");
            Ok(0)
        }
        Some(TaskCmd::Note { id, text }) => {
            c.s.task(id)?;
            c.s.event(Some(id), "note", &text);
            println!("T{id} noted");
            Ok(0)
        }
        Some(TaskCmd::Done { id, result, force }) => done(c, id, result, force),
        Some(TaskCmd::Fail { id, result }) => mark(id, "failed", result),
        Some(TaskCmd::Block { id, result }) => mark(id, "blocked", result),
        Some(TaskCmd::Cancel { id, result }) => mark(id, "cancelled", result),
        Some(TaskCmd::Reopen { id }) => mark(id, "todo", None),
    }
}
