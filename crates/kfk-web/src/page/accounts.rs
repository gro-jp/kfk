//! the account pool
use super::parts::*;
use crate::model::AccountRow;
use kfk_util::until;
use maud::{html, Markup};

pub fn accounts_page(v: &[AccountRow]) -> Markup {
    html! {
        h1 { "Accounts" }
        p.hint { "Usage is learned from what the agents and nodes report as they work; kfk makes no calls to ask. An account at its limit is skipped until its window resets." }
        @if v.is_empty() { (empty("The pool is empty: everything runs on the saved login. Add accounts with kfk auth.")) }
        @else {
            table {
                thead { tr { th scope="col" { "Slot" } th scope="col" { "Account" } th scope="col" { "In the pool" } th scope="col" { "Usage windows" } th scope="col" { "Used by" } th scope="col" { "Last seen" } } }
                tbody { @for a in v { tr {
                    th scope="row" .num { (a.account.slot) }
                    td { (a.account.label) }
                    td { (state_badge(if a.account.off { "off" } else { "on" })) }
                    td { @if a.windows.is_empty() { span.muted { "not seen yet" } } @else { ul.plain { @for w in &a.windows { li {
                        span.wname { (w.name) } " " @if w.full { (state_badge("limit")) } @else { (meter(w.util, &w.name)) } " " span.muted { "resets " (until(w.reset)) }
                    } } } } }
                    td { @for s in &a.seats { a.id href=(format!("/node/{s}")) { (s) } " " } @if a.agents > 0 { span.num { (a.agents) " agent" @if a.agents > 1 { "s" } } } @if a.seats.is_empty() && a.agents == 0 { span.muted { "–" } } }
                    td { (when(a.account.asof)) }
                } } }
            }
        }
    }
}
