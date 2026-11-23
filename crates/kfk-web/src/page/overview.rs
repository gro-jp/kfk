//! the overview: what needs the owner, the nodes, the agents, the board, the day's numbers
use super::parts::*;
use crate::lane::{Lane, BOARD};
use crate::model::{Card, NodeRow, Overview};
use kfk_report::Summary;
use kfk_util::{ktok, now};
use maud::{html, Markup};

pub fn overview_page(o: &Overview) -> Markup {
    html! {
        h1 { "Overview" }
        div #live hx-get="/part/overview" hx-trigger="every 4s" hx-swap="innerHTML" { (overview_part(o)) }
    }
}

pub fn overview_part(o: &Overview) -> Markup {
    html! {
        p.strip {
            "Fleet " (state_badge(if o.fleet_up { "up" } else { "down" }))
            " · mizan " (state_badge(if o.mizan { "on" } else { "off" }))
            " · agents " span.num { (o.agents.len()) " of " (o.max_agents) }
            " · project " code { (dash(&o.project)) }
        }
        section.needs aria-live="polite" aria-labelledby="h-needs" {
            h2 #h-needs { "Needs you" }
            @if o.needs.is_empty() { (empty("Nothing is waiting for you.")) }
            @else { ul.needlist { @for n in &o.needs { li { a.id href=(n.href) { (n.who) } " " (n.what) } } } }
        }
        section aria-labelledby="h-nodes" { h2 #h-nodes { "Nodes" } (nodes_table(&o.nodes)) }
        section aria-labelledby="h-agents" {
            h2 #h-agents { "Agents in flight" }
            @if o.agents.is_empty() { (empty("No agent is running.")) }
            @else {
                table {
                    thead { tr { th scope="col" { "Run" } th scope="col" { "Task" } th scope="col" { "Model" } th scope="col" { "Account" } th scope="col" { "Running for" } } }
                    tbody { @for a in &o.agents { tr {
                        td { a.id href=(format!("/run/{}", a.run.id)) { "run " (a.run.id) } }
                        td { (task_link(a.run.task)) " " (a.title) @if let Some(s) = a.stream { " " span.muted { "in " (task_link(s)) } } }
                        td { (a.run.model) "/" (a.run.effort) }
                        td.num { (crate::model::account_label_short(a.run.slot)) }
                        td.num { (dur(now() - a.run.started)) }
                    } } }
                }
            }
        }
        section aria-labelledby="h-board" { h2 #h-board { "Streams" } (board(&o.cards)) }
        section aria-labelledby="h-day" { h2 #h-day { "Last 24 hours" } (numbers(&o.day)) }
    }
}

pub fn nodes_table(nodes: &[NodeRow]) -> Markup {
    html! {
        @if nodes.is_empty() { (empty("No node has been started yet. Start the fleet with kfk up.")) }
        @else {
            table {
                thead { tr { th scope="col" { "Node" } th scope="col" { "State" } th scope="col" { "Model" } th scope="col" { "Account" } th scope="col" { "Account usage" } th scope="col" { "Context" } th scope="col" { "Running" } th scope="col" { "Queued" } th scope="col" { "Blocked" } } }
                tbody { @for n in nodes { tr {
                    th scope="row" { a.id href=(format!("/node/{}", n.seat.seat)) { (n.seat.seat) } }
                    td { (state_badge(&n.state)) @if !n.seat.note.is_empty() && n.state == "up" { " " span.note { (n.seat.note) } } }
                    td { (n.seat.model) "/" (n.seat.effort) }
                    td { (n.account) }
                    td { (meter(n.util, "account usage")) }
                    td { @match n.seat.ctx { Some(c) => { (meter(Some(100.0 * c as f64 / n.window.max(1) as f64), "context used before compaction")) " " span.muted { (ktok(c)) } }, None => { span.muted { "–" } } } }
                    td.num { (n.running) } td.num { (n.todo) } td.num { (n.blocked) }
                } } }
            }
        }
    }
}

pub fn card(c: &Card) -> Markup {
    html! {
        li.card.{"lane-" (c.lane.slug())} {
            div.card-top { (task_link(c.task.id)) " " span.muted { (dash(&c.task.owner)) } " " span.card-when { (when(Some(c.task.updated))) } }
            div.card-title { a href=(format!("/task/{}", c.task.id)) { (c.task.title) } }
            @if c.total > 0 {
                div.progress {
                    span.meter role="meter" aria-label="tasks done" aria-valuemin="0" aria-valuemax=(c.total) aria-valuenow=(c.done) { span.bar.m-ok style=(format!("width:{}%", 100 * c.done / c.total)) {} }
                    " " span.num { (c.done) "/" (c.total) } @if c.running > 0 { " · " span.num { (c.running) " running" } }
                }
            }
            @if c.lane == Lane::Stuck || c.lane == Lane::NeedsYou { @if !c.task.result.is_empty() { p.card-why { (c.task.result) } } }
        }
    }
}

pub fn board(cards: &[Card]) -> Markup {
    html! {
        @if cards.is_empty() { (empty("No stream yet. Give cos a goal.")) }
        @else {
            div.board {
                @for lane in BOARD {
                    @let mine: Vec<&Card> = cards.iter().filter(|c| c.lane == lane).collect();
                    section.col aria-label=(lane.label()) {
                        h3 { (lane_badge(lane)) " " span.count { (mine.len()) } }
                        p.hint { (lane.hint()) }
                        @if mine.is_empty() { p.empty { "–" } }
                        @else { ul.cards { @for c in mine.iter().take(12) { (card(c)) } } @if mine.len() > 12 { p.hint { (mine.len() - 12) " older not shown; see " a href="/history" { "History" } } } }
                    }
                }
            }
        }
    }
}

pub fn numbers(r: &Summary) -> Markup {
    let per = |x: Option<i64>| x.map(ktok).unwrap_or_else(|| "–".into());
    html! {
        div.stats {
            (stat("tasks passed", html! { (r.passed) }, &format!("{} on the first attempt, {} agent runs", r.first_try, r.runs)))
            (stat("weighted tokens", tok(r.total()), &format!("agents {}, nodes {}", ktok(r.agents), ktok(r.nodes))))
            (stat("per passed task", html! { (per(r.per_pass())) }, &format!("{} counting agents only", per(r.per_pass_agents()))))
            (stat("coordination share", html! { (r.coordination()) "%" }, "of all tokens went to cos and orchestrators"))
        }
        p.hint { "Weighted tokens are input-token equivalents: cache write 1.25×, cache read 0.1×, output 5×." }
    }
}
