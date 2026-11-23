//! what a node's pane is waiting for that no LLM can answer

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Wait {
    /// Claude asks whether the project folder is trusted: the owner's call, never kfk's
    Trust,
    /// first use of bypass-permissions mode on this machine
    Bypass,
    /// the account's token no longer works
    Login,
    /// the account hit a usage limit in mid-session
    Limit,
}

impl Wait {
    pub fn note(self) -> &'static str {
        match self {
            Wait::Trust => "waiting for the owner: accept the folder trust dialog (kfk attach)",
            Wait::Bypass => "waiting for the owner: accept bypass-permissions mode (kfk attach)",
            Wait::Login => "its account needs a new sign-in (kfk auth <slot>, then kfk swap)",
            Wait::Limit => "its account hit a usage limit",
        }
    }
}

/// read the visible tail of a pane
pub fn pane_wait(pane: &str) -> Option<Wait> {
    let lines: Vec<String> = pane.lines().filter(|l| !l.trim().is_empty()).map(|l| l.to_lowercase()).collect();
    let tail = lines[lines.len().saturating_sub(25)..].join("\n");
    let has = |x: &str| tail.contains(x);
    if has("trust this folder") {
        Some(Wait::Trust)
    } else if has("bypass permissions mode") && has("yes, i accept") {
        Some(Wait::Bypass)
    } else if has("select login method") || has("paste code here") || has("please run /login") {
        Some(Wait::Login)
    } else if has("hit your limit") || has("usage limit reached") || has("limit reached ·") {
        Some(Wait::Limit)
    } else {
        None
    }
}
