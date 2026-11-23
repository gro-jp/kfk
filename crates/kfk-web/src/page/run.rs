//! one agent attempt, and one node session
use super::log::log_part;
use super::parts::*;
use crate::model::{account_label_short, RunView, SessionView};
use kfk_util::now;
use maud::{html, Markup};

pub fn run_page(v: &RunView) -> Markup {
    let r = &v.run;
    html! {
        nav.crumbs aria-label="Breadcrumb" { a href="/" { "Overview" } " › " (task_link(r.task)) @if let Some(t) = &v.task { " " (t.title) } }
        h1 { "Run " (r.id) " " span.muted { "attempt " (r.attempt) " of T" (r.task) } }
        p.strip { (state_badge(&r.status)) " · " (r.model) "/" (r.effort) " · account " (account_label_short(r.slot)) " · " (dur(r.ended.unwrap_or_else(now) - r.started)) " · " (r.turns) " turns · " (tok(r.weighted())) " weighted" }
        dl.meta {
            dt { "Started" } dd { (clock(Some(r.started))) }
            dt { "Ended" } dd { (clock(r.ended)) }
            dt { "Check exit" } dd.num { @match r.check_rc { Some(rc) => (rc), None => "–" } }
            dt { "Work tree" } dd { code { (dash(&r.wd)) } }
            dt { "Tokens" } dd.num { "in " (r.in_tok) " · cache write " (r.cw_tok) " · cache read " (r.cr_tok) " · out " (r.out_tok) }
            dt { "Verdict" } dd { (dash(&r.note)) }
        }
        @if !v.prompt.is_empty() { details.block { summary { "Prompt the agent started from" } pre { (v.prompt.trim()) } } }
        (block("The agent's last words", &v.result))
        (block("Check output", &v.check))
        (log_part(&v.log, &format!("/run/{}", r.id)))
    }
}

pub fn session_page(v: &SessionView) -> Markup {
    let s = &v.session;
    html! {
        nav.crumbs aria-label="Breadcrumb" { a href="/nodes" { "Nodes" } " › " a href=(format!("/node/{}", s.seat)) { (s.seat) } }
        h1 { "Session of " (s.seat) " " span.muted.small { (s.sid) } }
        p.strip { (state_badge(if s.ended.is_none() { "running" } else { "ended" })) " · " (s.model) "/" (s.effort) " · account " (account_label_short(s.slot.unwrap_or(-1))) " · " (v.spend.turns) " turns · " (tok(v.spend.weighted())) " weighted" }
        dl.meta {
            dt { "Started" } dd { (clock(Some(s.started))) }
            dt { "Ended" } dd { (clock(s.ended)) }
            dt { "Tokens" } dd.num { "in " (v.spend.in_tok) " · cache write " (v.spend.cw_tok) " · cache read " (v.spend.cr_tok) " · out " (v.spend.out_tok) }
            dt { "File" } dd { code { (dash(&s.path)) } }
        }
        (log_part(&v.log, &format!("/session/{}", s.sid)))
    }
}
