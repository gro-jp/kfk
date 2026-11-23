//! delivery and token numbers over a period
use crate::ctx::Ctx;
use anyhow::Result;
use kfk_util::{ktok, now, stamp, table};

pub fn report(c: &Ctx, days: f64) -> Result<i32> {
    let r = kfk_report::summarise(&c.s, now() - 86400.0 * days, now())?;
    let per = |x: Option<i64>| x.map(ktok).unwrap_or_else(|| "-".into());
    println!("since {} ({days} days). Weighted = input-token equivalents (cache write 1.25x, cache read 0.1x, output 5x).\n", stamp(r.since, "%F %H:%M"));
    println!("tasks passed: {} ({} on the first attempt) of {} agent runs | weighted tokens: agents {}, nodes (cos and orchestrators) {}, total {}", r.passed, r.first_try, r.runs, ktok(r.agents), ktok(r.nodes), ktok(r.total()));
    println!("per passed task: {} all in ({} agents only) | coordination share {}% | agent cost as the CLI reports it (API-equivalent) ${:.2}", per(r.per_pass()), per(r.per_pass_agents()), r.coordination(), r.cost);
    if !r.spend.is_empty() {
        let rows: Vec<Vec<String>> = r.spend.iter().map(|x| vec![x.seat.clone(), x.turns.to_string(), ktok(x.weighted()), ktok(x.in_tok + x.cw_tok), ktok(x.cr_tok), ktok(x.out_tok)]).collect();
        println!("\n{}", table(&["node", "turns", "weighted", "input+write", "cache read", "output"], &rows));
    }
    if !r.rungs.is_empty() {
        let rows: Vec<Vec<String>> = r.rungs.iter().map(|a| vec![a.class.clone(), a.model.clone(), a.runs.to_string(), a.pass.to_string(), a.fail.to_string(), a.limit.to_string(), format!("{}%", a.pass_rate()), ktok(a.weighted / a.runs), format!("{}m{:02}s", a.secs / a.runs / 60, a.secs / a.runs % 60)]).collect();
        println!("\n{}", table(&["class", "model/effort", "runs", "pass", "fail", "limit", "pass rate", "weighted/run", "avg time"], &rows));
    }
    if !r.streams.is_empty() {
        let rows: Vec<Vec<String>> = r.streams.iter().map(|s| vec![format!("T{}", s.id), s.status.clone(), if s.owner.is_empty() { "-".into() } else { s.owner.clone() }, format!("{}/{}", s.done, s.children), ktok(s.weighted), s.title.chars().take(60).collect()]).collect();
        println!("\n{}", table(&["stream", "status", "owner", "children", "weighted", "title"], &rows));
    }
    Ok(0)
}
