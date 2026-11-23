//! small pieces every page uses
use crate::lane::Lane;
use kfk_util::{ago, ktok, now, stamp};
use maud::{html, Markup};

pub fn lane_badge(l: Lane) -> Markup {
    html! { span.badge.{"lane-" (l.slug())} title=(l.hint()) { span.mark aria-hidden="true" { (l.mark()) } " " (l.label()) } }
}

/// a state word (up, pass, fail, running, …) with a mark, so colour is never the only signal
pub fn state_badge(state: &str) -> Markup {
    let (class, mark) = match state {
        "up" | "pass" | "done" | "on" => ("ok", "✓"),
        "running" => ("go", "▶"),
        "fail" | "failed" | "dead" | "error" | "lost" => ("bad", "!"),
        "limit" | "blocked" | "down" => ("warn", "!"),
        "todo" => ("wait", "…"),
        _ => ("off", "–"),
    };
    html! { span.badge.{"st-" (class)} { span.mark aria-hidden="true" { (mark) } " " (state) } }
}

/// a percentage as a bar with its number beside it
pub fn meter(pct: Option<f64>, what: &str) -> Markup {
    match pct {
        None => html! { span.muted { "unknown" } },
        Some(p) => {
            let p = p.clamp(0.0, 100.0);
            let level = if p >= 95.0 { "bad" } else if p >= 80.0 { "warn" } else { "ok" };
            html! {
                span.meter role="meter" aria-label=(what) aria-valuemin="0" aria-valuemax="100" aria-valuenow=(format!("{p:.0}")) {
                    span.bar.{"m-" (level)} style=(format!("width:{p:.0}%")) {}
                }
                " " span.num { (format!("{p:.0}%")) }
            }
        }
    }
}

pub fn when(ts: Option<f64>) -> Markup {
    match ts.filter(|t| *t > 0.0) {
        None => html! { span.muted { "–" } },
        Some(t) => html! { time datetime=(stamp(t, "%Y-%m-%dT%H:%M:%S%z")) title=(stamp(t, "%a %d %b %Y %H:%M:%S")) { (ago(Some(t), now())) " ago" } },
    }
}

pub fn clock(ts: Option<f64>) -> Markup {
    match ts.filter(|t| *t > 0.0) {
        None => html! { span.muted { "–" } },
        Some(t) => html! { time datetime=(stamp(t, "%Y-%m-%dT%H:%M:%S%z")) title=(stamp(t, "%a %d %b %Y %H:%M:%S")) { (stamp(t, "%d %b %H:%M")) } },
    }
}

pub fn dur(secs: f64) -> String {
    let s = secs.max(0.0) as i64;
    if s >= 3600 { format!("{}h {:02}m", s / 3600, s % 3600 / 60) } else { format!("{}m {:02}s", s / 60, s % 60) }
}

pub fn tok(n: i64) -> Markup {
    html! { span.num title=(format!("{n} weighted tokens")) { (ktok(n)) } }
}

pub fn task_link(id: i64) -> Markup {
    html! { a.id href=(format!("/task/{id}")) { "T" (id) } }
}

pub fn dash(s: &str) -> &str {
    if s.trim().is_empty() { "–" } else { s }
}

/// a labelled block of preformatted text; nothing when the text is empty
pub fn block(title: &str, text: &str) -> Markup {
    html! { @if !text.trim().is_empty() { section.block { h3 { (title) } pre { (text.trim()) } } } }
}

pub fn stat(label: &str, value: Markup, note: &str) -> Markup {
    html! { div.stat { div.stat-v { (value) } div.stat-l { (label) } @if !note.is_empty() { div.stat-n { (note) } } } }
}

pub fn empty(text: &str) -> Markup {
    html! { p.empty { (text) } }
}
