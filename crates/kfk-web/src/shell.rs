//! a terminal on a node: whether a page may show one, and whether a request may open one
use crate::probe::Probe;
use anyhow::{anyhow, Result};
use kfk_store::Store;
use kfk_term::{admit, attach_args, seat_ok, usable, Ask};

/// can this server offer terminals? the reason when it cannot
pub fn shell_usable(s: &Store, local: bool) -> Result<(), &'static str> {
    usable(s.cfg("web_shell") != "off", local).map_err(|d| d.why())
}

pub struct ShellView {
    pub seat: String,
    /// up | dead | down | stopped
    pub state: String,
    pub usable: Result<(), &'static str>,
}

/// the page that holds nothing but one node's terminal
pub fn shell_view(s: &Store, p: &dyn Probe, local: bool, name: &str) -> Result<ShellView> {
    let seat = s.seat(name)?.ok_or_else(|| anyhow!("no node {name}"))?;
    Ok(ShellView { state: p.node_state(s, &seat), seat: seat.seat, usable: shell_usable(s, local) })
}

pub struct Open<'a> {
    pub local: bool,
    pub seat: &'a str,
    pub host: Option<&'a str>,
    pub origin: Option<&'a str>,
    /// the tmux socket name, when the fleet is not on the default server
    pub socket: Option<&'a str>,
    /// tells this terminal's tmux session from the others
    pub id: &'a str,
}

/// the tmux command for a terminal request, or the HTTP status and the reason it is refused
pub fn shell_open(s: &Store, p: &dyn Probe, o: &Open) -> Result<Vec<String>, (u16, String)> {
    admit(&Ask { enabled: s.cfg("web_shell") != "off", local: o.local, host: o.host, origin: o.origin }).map_err(|d| (403, d.why().to_string()))?;
    let seat = Some(o.seat).filter(|n| seat_ok(n)).and_then(|n| s.seat(n).ok().flatten()).ok_or_else(|| (404, format!("There is no node {}.", o.seat)))?;
    match p.node_state(s, &seat).as_str() {
        "up" | "dead" => Ok(attach_args(o.socket, &s.cfg("tmux_session"), &seat.seat, o.id)),
        state => Err((409, format!("{} is {state}, so it has no terminal. Start the fleet with kfk up; cos starts orchestrators.", seat.seat))),
    }
}
