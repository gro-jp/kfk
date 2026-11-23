//! the numbers of a period and the event log
use super::overview::numbers;
use super::parts::*;
use crate::model::History;
use kfk_util::ktok;
use maud::{html, Markup};

pub fn history_page(h: &History) -> Markup {
    let r = &h.summary;
    html! {
        h1 { "History" }
        nav.pager aria-label="Period" { "Period: " @for d in [1.0, 7.0, 30.0, 365.0] { a href=(format!("/history?days={d}")) aria-current=[(d == h.days).then_some("true")] { (d) " day" @if d > 1.0 { "s" } } " " } }
        (numbers(r))
        @if !r.spend.is_empty() {
            section aria-labelledby="h-nspend" { h2 #h-nspend { "Node spend" }
                table {
                    thead { tr { th scope="col" { "Node" } th scope="col" { "Turns" } th scope="col" { "Weighted" } th scope="col" { "Input + cache write" } th scope="col" { "Cache read" } th scope="col" { "Output" } } }
                    tbody { @for x in &r.spend { tr { th scope="row" { a.id href=(format!("/node/{}", x.seat)) { (x.seat) } } td.num { (x.turns) } td { (tok(x.weighted())) } td.num { (ktok(x.in_tok + x.cw_tok)) } td.num { (ktok(x.cr_tok)) } td.num { (ktok(x.out_tok)) } } } }
                }
            }
        }
        @if !r.rungs.is_empty() {
            section aria-labelledby="h-rungs" { h2 #h-rungs { "Agent runs by class and model" }
                table {
                    thead { tr { th scope="col" { "Class" } th scope="col" { "Model/effort" } th scope="col" { "Runs" } th scope="col" { "Pass" } th scope="col" { "Fail" } th scope="col" { "Limit" } th scope="col" { "Pass rate" } th scope="col" { "Weighted per run" } th scope="col" { "Average time" } } }
                    tbody { @for a in &r.rungs { tr { th scope="row" { (a.class) } td { (a.model) } td.num { (a.runs) } td.num { (a.pass) } td.num { (a.fail) } td.num { (a.limit) } td.num { (a.pass_rate()) "%" } td { (tok(a.weighted / a.runs.max(1))) } td.num { (dur((a.secs / a.runs.max(1)) as f64)) } } } }
                }
            }
        }
        @if !r.streams.is_empty() {
            section aria-labelledby="h-streams" { h2 #h-streams { "Streams touched in this period" }
                table {
                    thead { tr { th scope="col" { "Stream" } th scope="col" { "Status" } th scope="col" { "Owner" } th scope="col" { "Tasks done" } th scope="col" { "Weighted" } th scope="col" { "Title" } } }
                    tbody { @for s in &r.streams { tr { th scope="row" { (task_link(s.id)) } td { (state_badge(&s.status)) } td { (dash(&s.owner)) } td.num { (s.done) "/" (s.children) } td { (tok(s.weighted)) } td { a href=(format!("/task/{}", s.id)) { (s.title) } } } } }
                }
            }
        }
        section aria-labelledby="h-events" { h2 #h-events { "Events, newest first" }
            @if h.events.is_empty() { (empty("Nothing has happened yet.")) }
            @else { ol.events { @for (ts, task, actor, kind, data) in &h.events { li { (clock(Some(*ts))) " " span.badge.st-off { (kind) } " " span.muted { (actor) } " " @if let Some(t) = task { (task_link(*t)) " " } (data) } } } }
        }
    }
}
