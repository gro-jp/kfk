//! the tmux command behind one web terminal

/// a node name that is safe inside a tmux target: `cos`, `o3`
pub fn seat_ok(seat: &str) -> bool {
    !seat.is_empty() && seat.len() <= 16 && seat.chars().all(|c| c.is_ascii_alphanumeric())
}

/// a tmux client with a session of its own in the fleet's group: it shares the fleet's windows, but which window
/// it shows, its size, its mouse setting and its status line are its own, so `kfk attach` is not disturbed.
/// The session goes away with its client, and when it would show any other window.
pub fn attach_args(socket: Option<&str>, session: &str, seat: &str, id: &str) -> Vec<String> {
    let mut a: Vec<String> = vec![];
    if let Some(s) = socket {
        a.extend(["-L".into(), s.into()]);
    }
    let own = format!("{session}-web-{id}");
    for part in [
        vec!["new-session", "-t", session, "-s", &own],
        vec!["set-option", "destroy-unattached", "on"],
        vec!["set-option", "mouse", "on"],
        vec!["set-option", "status", "off"],
        vec!["select-window", "-t", &format!(":={seat}")],
        // from here on the session shows this node only: if its window goes (a restart replaces it) or the
        // viewer switches windows, the terminal ends instead of quietly typing into another node
        vec!["set-hook", "session-window-changed", "kill-session"],
    ] {
        if a.iter().any(|x| x == "new-session") {
            a.push(";".into());
        }
        a.extend(part.into_iter().map(String::from));
    }
    a
}
