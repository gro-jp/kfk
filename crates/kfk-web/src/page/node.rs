//! the nodes, and one node drilled into
use super::overview::nodes_table;
use super::parts::*;
use super::term::term_part;
use crate::model::{account_label_short, NodeRow, NodeView};
use kfk_store::Spend;
use kfk_util::{ktok, stamp};
use maud::{html, Markup};

pub fn nodes_page(v: &[(NodeRow, Spend)]) -> Markup {
    html! {
        h1 { "Nodes" }
        p.hint { "cos and the orchestrators it started. mizan picks each node's account, restarts a dead node, moves a node off an account near its limit, and stops an orchestrator that has been idle." }
        @if v.is_empty() { (empty("No node has been started yet.")) }
        @else {
            table {
                thead { tr { th scope="col" { "Node" } th scope="col" { "Role" } th scope="col" { "State" } th scope="col" { "Model" } th scope="col" { "Account" } th scope="col" { "Turns, all time" } th scope="col" { "Weighted, all time" } th scope="col" { "Since" } } }
                tbody { @for (n, sp) in v { tr {
                    th scope="row" { a.id href=(format!("/node/{}", n.seat.seat)) { (n.seat.seat) } }
                    td { (n.seat.role) } td { (state_badge(&n.state)) } td { (n.seat.model) "/" (n.seat.effort) } td { (n.account) }
                    td.num { (sp.turns) } td { (tok(sp.weighted())) } td { (when(n.seat.since)) }
                } } }
            }
        }
    }
}

pub fn node_page(v: &NodeView) -> Markup {
    let name = &v.row.seat.seat;
    html! {
        nav.crumbs aria-label="Breadcrumb" { a href="/nodes" { "Nodes" } }
        h1 { (name) " " span.muted.small { @if v.row.seat.role == "cos" { "chief of staff" } @else { "orchestrator" } } }
        @if v.term.is_ok() { (term_part(name, v.row.state == "up", false)) }
        div #live hx-get=(format!("/part/node/{name}")) hx-trigger="every 3s" hx-swap="innerHTML" { (node_part(v)) }
        section aria-labelledby="h-sess" {
            h2 #h-sess { "Sessions" }
            @if v.sessions.is_empty() { (empty("No session recorded for this node yet.")) }
            @else {
                table {
                    thead { tr { th scope="col" { "Session" } th scope="col" { "State" } th scope="col" { "Started" } th scope="col" { "Ended" } th scope="col" { "Model" } th scope="col" { "Account" } th scope="col" { "Turns" } th scope="col" { "Weighted" } } }
                    tbody { @for (s, sp) in &v.sessions { tr {
                        th scope="row" { a.id href=(format!("/session/{}", s.sid)) { (s.sid.get(..8).unwrap_or(&s.sid)) "…" } }
                        td { (state_badge(if s.ended.is_none() { "running" } else { "ended" })) }
                        td { (clock(Some(s.started))) } td { (clock(s.ended)) } td { (s.model) "/" (s.effort) }
                        td.num { (account_label_short(s.slot.unwrap_or(-1))) } td.num { (sp.turns) } td { (tok(sp.weighted())) }
                    } } }
                }
            }
        }
    }
}

pub fn node_part(v: &NodeView) -> Markup {
    let peak = v.hours.iter().map(|h| h.1).max().unwrap_or(0).max(1);
    html! {
        (nodes_table(std::slice::from_ref(&v.row)))
        @if let Err(why) = v.term {
            section aria-labelledby="h-now" {
                h2 #h-now { "What it is doing now" }
                @if v.pane.is_empty() { (empty("The node is not up, so there is no screen to show.")) }
                @else { pre.pane aria-label="The last lines of the node's screen" { (v.pane) } }
                p.hint { (why) }
            }
        }
        section aria-labelledby="h-queue" {
            h2 #h-queue { "Open work it owns" }
            @if v.queue.is_empty() { (empty("Nothing open.")) }
            @else { ul.plain { @for c in &v.queue { li { (lane_badge(c.lane)) " " (task_link(c.task.id)) " " a href=(format!("/task/{}", c.task.id)) { (c.task.title) } @if c.total > 0 { " " span.muted { (c.done) "/" (c.total) } } } } } }
        }
        section aria-labelledby="h-spend" {
            h2 #h-spend { "Tokens by hour, last 24 hours" }
            figure.hours {
                div.bars role="img" aria-label=(format!("Weighted tokens per hour; the highest hour is {}", ktok(peak))) {
                    @for (h, w) in &v.hours { span.hbar title=(format!("{}: {}", stamp(*h as f64 * 3600.0, "%d %b %H:00"), ktok(*w))) style=(format!("height:{}%", (100 * w / peak).max(if *w > 0 { 3 } else { 0 }))) {} }
                }
                figcaption { "Total " (ktok(v.hours.iter().map(|h| h.1).sum())) " weighted tokens; highest hour " (ktok(peak)) "." }
            }
        }
    }
}
