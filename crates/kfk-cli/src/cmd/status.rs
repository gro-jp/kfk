//! the fleet at a glance
use crate::ctx::{slot_name, Ctx};
use anyhow::Result;
use kfk_usage::{util, LOGIN};
use kfk_util::{ago, alive, ktok, now, table, weighted};

pub fn status(c: &Ctx, json: bool) -> Result<i32> {
    c.s.reconcile(&alive)?;
    let (seats, running) = (c.s.seats()?, c.s.running()?);
    if json {
        let blocked = c.s.tasks(&kfk_store::TaskFilter { status: Some("blocked".into()), limit: 200, ..Default::default() })?;
        println!("{}", serde_json::json!({"fleet_up": kfk_mizan::session_up(&c.s), "mizan": kfk_mizan::mizan_pid(&c.s.kh), "seats": seats, "running": running, "blocked": blocked}));
        return Ok(0);
    }
    println!("fleet: {} | mizan: {} | project: {} | data: {}", if kfk_mizan::session_up(&c.s) { "up" } else { "down" }, if kfk_mizan::mizan_pid(&c.s.kh).is_some() { "on" } else { "off" }, c.s.cfg("project_dir"), c.s.kh);
    let mut rows = vec![];
    for s in &seats {
        let u = c.s.account(s.slot.unwrap_or(LOGIN))?.and_then(|a| util(&a.wins, &s.model, now()).0);
        let (r, t, b) = c.s.open_counts(&s.seat)?;
        rows.push(vec![s.seat.clone(), kfk_mizan::node_state(&c.s, s).to_string(), slot_name(s.slot), u.map(|u| format!("{}%", u as i64)).unwrap_or_else(|| "?".into()), format!("{}/{}", s.model, s.effort),
                       s.ctx.map(ktok).unwrap_or_else(|| "-".into()), r.to_string(), t.to_string(), b.to_string()]);
    }
    println!("{}", table(&["seat", "node", "slot", "usage", "model", "ctx", "running", "todo", "blocked"], &rows));
    for s in seats.iter().filter(|s| !s.note.is_empty() && s.state == "up") {
        println!("{}: {}", s.seat, s.note);
    }
    println!("\nagents running: {} of {}", running.len(), c.s.cfgi("max_agents"));
    for r in &running {
        let title: String = c.s.task(r.task).map(|t| t.title).unwrap_or_default().chars().take(60).collect();
        println!("  T{} run {} | {}/{} slot {} | {} | {title}", r.task, r.id, r.model, r.effort, slot_name(Some(r.slot)), ago(Some(r.started), now()));
    }
    let need: Vec<_> = c.s.tasks(&kfk_store::TaskFilter { status: Some("blocked".into()), limit: 20, ..Default::default() })?;
    for t in &need {
        println!("blocked T{} ({}): {}", t.id, if t.owner.is_empty() { "-" } else { &t.owner }, t.result);
    }
    let day = c.s.runs_since(now() - 86400.0)?;
    let nodes: i64 = c.s.spend_since(now() - 86400.0)?.iter().map(|x| x.weighted()).sum();
    if !day.is_empty() || nodes > 0 {
        let w: i64 = day.iter().map(|r| weighted(r.in_tok, r.cw_tok, r.cr_tok, r.out_tok)).sum();
        let pass = day.iter().filter(|r| r.status == "pass").count();
        println!("last 24h: {} agent runs, {pass} passed | weighted tokens: agents {}, nodes {}{}", day.len(), ktok(w), ktok(nodes), if pass > 0 { format!(" | {} per passed task, all in", ktok((w + nodes) / pass as i64)) } else { String::new() });
    }
    Ok(0)
}
