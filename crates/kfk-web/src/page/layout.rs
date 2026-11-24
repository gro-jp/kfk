//! the frame around every page
use maud::{html, Markup, DOCTYPE};

pub const NAV: [(&str, &str); 4] = [("/", "Overview"), ("/nodes", "Nodes"), ("/accounts", "Accounts"), ("/history", "History")];

/// `here` is the nav entry this page belongs to
pub fn layout(title: &str, here: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="color-scheme" content="light dark";
                title { (title) " · kfk" }
                link rel="stylesheet" href="/assets/app.css";
                script src="/assets/htmx.min.js" defer {}
            }
            body {
                a.skip href="#main" { "Skip to content" }
                header.top {
                    a.brand href="/" { "kfk" span.brand-sub { "kun fayakoon" } }
                    nav aria-label="Main" {
                        ul { @for (href, name) in NAV { li { a href=(href) aria-current=[(href == here).then_some("page")] { (name) } } } }
                    }
                }
                main #main tabindex="-1" { (body) }
                footer { "The pages only show; a node's terminal types into its live session. In a terminal, talk to cos with " code { "kfk attach" } "." }
            }
        }
    }
}

pub fn error_page(status: u16, what: &str) -> Markup {
    layout("Not here", "", html! {
        h1 { @if status == 404 { "Not found" } @else { "Something failed" } }
        p { (what) }
        p { a href="/" { "Back to the overview" } }
    })
}
