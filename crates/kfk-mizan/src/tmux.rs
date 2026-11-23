//! the tmux session that holds the nodes
use crate::prelude::*;

pub fn tmux(args: &[&str]) -> Output {
    let mut c = Command::new("tmux");
    if let Ok(s) = std::env::var("KFK_TMUX_SOCKET") {
        c.args(["-L", &s]);
    }
    c.args(args).output().unwrap_or_else(|_| Command::new("false").output().expect("false"))
}

pub fn session_up(s: &Store) -> bool {
    tmux(&["has-session", "-t", &format!("={}", s.cfg("tmux_session"))]).status.success()
}

pub fn window_up(s: &Store, seat: &str) -> bool {
    tmux(&["list-panes", "-t", &format!("={}:={}", s.cfg("tmux_session"), seat)]).status.success()
}

/// up (claude alive in its window) | dead (window there, claude gone) | down (no window) | stopped (by design)
pub fn node_state(s: &Store, seat: &Seat) -> &'static str {
    if seat.state != "up" {
        "stopped"
    } else if !window_up(s, &seat.seat) {
        "down"
    } else if seat.pid.map(alive).unwrap_or(false) {
        "up"
    } else {
        "dead"
    }
}

/// the visible text of a node's pane
pub fn capture(s: &Store, seat: &str) -> String {
    let o = tmux(&["capture-pane", "-p", "-t", &format!("={}:={}", s.cfg("tmux_session"), seat)]);
    String::from_utf8_lossy(&o.stdout).into_owned()
}
