//! one task: its brief, its children, its attempts, its history
use super::parts::*;
use crate::model::TaskView;
use kfk_store::Run;
use kfk_util::now;
use maud::{html, Markup};

pub fn task_page(v: &TaskView) -> Markup {
    let t = &v.card.task;
    html! {
        nav.crumbs aria-label="Breadcrumb" { a href="/" { "Overview" } @if let Some(p) = &v.parent { " › " (task_link(p.id)) " " (p.title) } }
        h1 { "T" (t.id) " " (t.title) }
        div #live hx-get=(format!("/part/task/{}", t.id)) hx-trigger="every 5s" hx-swap="innerHTML" { (task_part(v)) }
        (block("Spec", &t.spec))
        (block("Done when", &t.dod))
        (block("Check (exit status 0 = done)", &t.chk))
    }
}

pub fn runs_table(runs: &[Run]) -> Markup {
    html! {
        table {
            thead { tr { th scope="col" { "Run" } th scope="col" { "Try" } th scope="col" { "Outcome" } th scope="col" { "Model" } th scope="col" { "Account" } th scope="col" { "Time" } th scope="col" { "Turns" } th scope="col" { "Weighted" } th scope="col" { "Note" } } }
            tbody { @for r in runs { tr {
                th scope="row" { a.id href=(format!("/run/{}", r.id)) { "run " (r.id) } }
                td.num { (r.attempt) }
                td { (state_badge(&r.status)) }
                td { (r.model) "/" (r.effort) }
                td.num { (crate::model::account_label_short(r.slot)) }
                td.num { (dur(r.ended.unwrap_or_else(now) - r.started)) }
                td.num { (r.turns) }
                td { (tok(r.weighted())) }
                td { (r.note) }
            } } }
        }
    }
}

pub fn task_part(v: &TaskView) -> Markup {
    let t = &v.card.task;
    html! {
        p.strip { (lane_badge(v.card.lane)) " " span.muted { "status " (t.status) } " · updated " (when(Some(t.updated))) }
        dl.meta {
            dt { "Owner" } dd { @if t.owner.is_empty() { "–" } @else { a href=(format!("/node/{}", t.owner)) { (t.owner) } } }
            dt { "Class" } dd { (t.class) @if !t.model.is_empty() { " (" (t.model) "/" (dash(&t.effort)) ")" } }
            dt { "Attempts" } dd.num { (t.attempts) }
            dt { "Depends on" } dd { @if t.dep_ids().is_empty() { "–" } @else { @for d in t.dep_ids() { (task_link(d)) " " } } }
            dt { "Repository" } dd { code { (dash(&t.repo)) } }
            dt { "Created" } dd { (clock(Some(t.created))) }
        }
        (block("Result", &t.result))
        @if !v.kids.is_empty() {
            section aria-labelledby="h-kids" {
                h2 #h-kids { "Tasks " span.count { (v.card.done) "/" (v.card.total) " done" } }
                table {
                    thead { tr { th scope="col" { "Task" } th scope="col" { "State" } th scope="col" { "Title" } th scope="col" { "Class" } th scope="col" { "Tries" } th scope="col" { "Updated" } th scope="col" { "Result" } } }
                    tbody { @for k in &v.kids { tr {
                        th scope="row" { (task_link(k.task.id)) }
                        td { (lane_badge(k.lane)) }
                        td { a href=(format!("/task/{}", k.task.id)) { (k.task.title) } }
                        td { (k.task.class) } td.num { (k.task.attempts) } td { (when(Some(k.task.updated))) } td { (k.task.result) }
                    } } }
                }
            }
        }
        @if !v.runs.is_empty() { section aria-labelledby="h-runs" { h2 #h-runs { "Attempts" } (runs_table(&v.runs)) } }
        (block("Check output of the last attempt (tail)", &v.check_tail))
        (block("The agent's last words", &v.last_words))
        @if !v.events.is_empty() {
            section aria-labelledby="h-ev" {
                h2 #h-ev { "History" }
                ol.events { @for (ts, actor, kind, data) in &v.events { li { (clock(Some(*ts))) " " span.badge.st-off { (kind) } " " span.muted { (actor) } " " (data) } } }
            }
        }
    }
}
