//! a transcript, one page at a time
use crate::source::Log;
use crate::transcript::{clock, Entry};
use maud::{html, Markup};

fn entry(e: &Entry) -> Markup {
    let head = html! { span.e-time { (clock(&e.ts)) } " " span.e-kind { (e.kind.label()) } @if !e.title.is_empty() { " " span.e-title { (e.title) } } @if e.error { " " span.badge.st-bad { "! error" } } };
    html! {
        li.entry.{"k-" (e.kind.slug())} {
            @if e.kind.folded() && !e.body.is_empty() { details { summary { (head) " " span.muted { (e.body.lines().count()) " lines" } } pre { (e.body) } } }
            @else { div.e-head { (head) } @if !e.body.is_empty() { pre { (e.body) } } }
        }
    }
}

/// `base` is the page's own path; the pager keeps the `all` choice
pub fn log_part(l: &Log, base: &str) -> Markup {
    let p = l.paging;
    let link = |page: usize, all: bool| format!("{base}?page={page}{}", if all { "&all=1" } else { "" });
    html! {
        section.log aria-labelledby="h-log" {
            h2 #h-log { "Transcript" }
            p.hint {
                (p.total) " lines, read from " (l.origin) ". "
                @if l.all { a href=(link(p.page, false)) { "Hide bookkeeping lines" } } @else { a href=(link(p.page, true)) { "Show bookkeeping lines too" } }
            }
            @if p.pages > 1 {
                nav.pager aria-label="Transcript pages" {
                    @if p.page > 1 { a href=(link(1, l.all)) { "« Oldest" } " " a rel="prev" href=(link(p.page - 1, l.all)) { "‹ Older" } }
                    " " span { "Page " (p.page) " of " (p.pages) } " "
                    @if p.page < p.pages { a rel="next" href=(link(p.page + 1, l.all)) { "Newer ›" } " " a href=(link(p.pages, l.all)) { "Newest »" } }
                }
            }
            @if l.entries.is_empty() { p.empty { "Nothing to show on this page." } }
            @else { ol.entries { @for e in &l.entries { (entry(e)) } } }
        }
    }
}
