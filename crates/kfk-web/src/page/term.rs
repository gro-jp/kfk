//! the terminal component, and the page that holds only a terminal
use crate::shell::ShellView;
use maud::{html, Markup, DOCTYPE};

/// a live terminal on `seat`. `up`: connect at once. `alone`: it is the whole page.
pub fn term_part(seat: &str, up: bool, alone: bool) -> Markup {
    html! {
        section.term.term-max[alone] data-term=(seat) data-auto=(if up { "1" } else { "0" }) aria-labelledby="h-term" {
            div.term-bar {
                h2 #h-term { "Terminal" @if alone { " · " (seat) } }
                span.term-status role="status" { @if up { "Starting…" } @else { "The node is not up." } }
                button type="button" data-term-connect hidden[up] { "Connect" }
                @if alone { a href=(format!("/node/{seat}")) { "Back to " (seat) } }
                @else { button type="button" data-term-full aria-pressed="false" { "Full screen" } a href=(format!("/shell/{seat}")) { "Open alone" } }
            }
            div.term-screen {}
            p.hint { "This is the node's live session: what you type goes to it. The mouse wheel scrolls back through its history; scroll back down to the bottom, or press q, to type again. To select text, hold Option (macOS) or Shift while dragging." }
            noscript { p.empty { "The terminal needs JavaScript." } }
            link rel="stylesheet" href="/assets/xterm.css";
            script src="/assets/xterm.js" {}
            script src="/assets/xterm-addon-fit.js" {}
            script src="/assets/term.js" {}
        }
    }
}

pub fn shell_page(v: &ShellView) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="color-scheme" content="light dark";
                title { (v.seat) " terminal · kfk" }
                link rel="stylesheet" href="/assets/app.css";
            }
            body {
                @match v.usable {
                    Ok(()) => { main { (term_part(&v.seat, matches!(v.state.as_str(), "up" | "dead"), true)) } }
                    Err(why) => { main { h1 { (v.seat) " terminal" } p.empty { (why) } p { a href=(format!("/node/{}", v.seat)) { "Back to " (v.seat) } } } }
                }
            }
        }
    }
}
