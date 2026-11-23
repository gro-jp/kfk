//! everything about one task
use crate::ctx::{slot_name, Ctx};
use anyhow::Result;
use kfk_store::Run;
use kfk_util::{ktok, now, stamp, tail_lines};

pub fn run_line(x: &Run) -> String {
    let dur = (x.ended.unwrap_or_else(now) - x.started).max(0.0) as i64;
    format!("  run {} try {}: {} | {}/{} slot {} | {}m{:02}s | {} turns | out {}, weighted {} | {}", x.id, x.attempt, x.status.to_uppercase(), x.model, x.effort, slot_name(Some(x.slot)),
            dur / 60, dur % 60, x.turns, ktok(x.out_tok), ktok(x.weighted()), if x.note.is_empty() { &x.dir } else { &x.note })
}

pub fn show(c: &Ctx, id: i64, json: bool) -> Result<i32> {
    let t = c.s.task(id)?;
    let runs = c.s.runs_of(id)?;
    if json {
        let mut v = serde_json::to_value(&t)?;
        v["runs"] = serde_json::to_value(&runs)?;
        println!("{v}");
        return Ok(0);
    }
    let dash = |x: &str| if x.is_empty() { "-".to_string() } else { x.to_string() };
    println!("T{} [{}] {}\nowner {} | class {} | tries {} | stream {} | deps {} | repo {}", t.id, t.status, t.title, dash(&t.owner), t.class, t.attempts, t.parent.map(|p| format!("T{p}")).unwrap_or_else(|| "-".into()), dash(&t.deps), dash(&t.repo));
    for (label, v) in [("SPEC", &t.spec), ("DONE WHEN", &t.dod), ("CHECK", &t.chk), ("RESULT", &t.result)] {
        if !v.trim().is_empty() {
            println!("{label}: {}", v.trim());
        }
    }
    let kids = c.s.children(id)?;
    if !kids.is_empty() {
        let mut n = std::collections::BTreeMap::new();
        for k in &kids {
            *n.entry(k.status.clone()).or_insert(0) += 1;
        }
        println!("CHILDREN: {}", n.iter().map(|(s, k)| format!("{k} {s}")).collect::<Vec<_>>().join(", "));
    }
    for r in &runs {
        println!("{}", run_line(r));
    }
    if let Some(last) = runs.last().filter(|_| t.status != "done") {
        for (file, label) in [("check.log", "check output"), ("result.md", "agent's last words")] {
            let text = std::fs::read_to_string(format!("{}/{file}", last.dir)).unwrap_or_default();
            if !text.trim().is_empty() {
                println!("--- {label} (tail) ---\n{}", tail_lines(text.trim(), 25, 2500));
            }
        }
    }
    for (ts, actor, _, data) in c.s.events(Some(id), &["note"], 50)? {
        println!("NOTE {} {actor}: {data}", stamp(ts, "%m-%d %H:%M"));
    }
    Ok(0)
}
