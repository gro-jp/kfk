//! where a piece of work stands, in the owner's words
use kfk_store::Task;

#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
pub enum Lane {
    NeedsYou,
    Stuck,
    InFlight,
    Queued,
    Planned,
    Done,
    Dropped,
}

pub const BOARD: [Lane; 6] = [Lane::NeedsYou, Lane::Stuck, Lane::InFlight, Lane::Queued, Lane::Planned, Lane::Done];

impl Lane {
    pub fn label(self) -> &'static str {
        match self {
            Lane::NeedsYou => "Needs you",
            Lane::Stuck => "Stuck",
            Lane::InFlight => "In flight",
            Lane::Queued => "Queued",
            Lane::Planned => "Planned",
            Lane::Done => "Done",
            Lane::Dropped => "Dropped",
        }
    }
    pub fn slug(self) -> &'static str {
        match self {
            Lane::NeedsYou => "needs",
            Lane::Stuck => "stuck",
            Lane::InFlight => "flight",
            Lane::Queued => "queued",
            Lane::Planned => "planned",
            Lane::Done => "done",
            Lane::Dropped => "dropped",
        }
    }
    /// one line that says what the lane means
    pub fn hint(self) -> &'static str {
        match self {
            Lane::NeedsYou => "waiting for a decision only you can make",
            Lane::Stuck => "failed or blocked; an orchestrator or cos has to act",
            Lane::InFlight => "agents are working on it now",
            Lane::Queued => "cut into tasks, waiting for a free agent or a dependency",
            Lane::Planned => "handed to an orchestrator, not yet cut into tasks",
            Lane::Done => "its check passed",
            Lane::Dropped => "cancelled",
        }
    }
    /// a mark that carries the state without colour
    pub fn mark(self) -> &'static str {
        match self {
            Lane::NeedsYou => "?",
            Lane::Stuck => "!",
            Lane::InFlight => "▶",
            Lane::Queued => "…",
            Lane::Planned => "○",
            Lane::Done => "✓",
            Lane::Dropped => "×",
        }
    }
}

fn asks_owner(t: &Task) -> bool {
    t.status == "blocked" && t.result.to_lowercase().contains("needs owner")
}

/// the lane of a task; a stream takes the most urgent lane among itself and its children
pub fn lane(t: &Task, kids: &[Task]) -> Lane {
    let open: Vec<&Task> = kids.iter().filter(|k| k.status != "cancelled").collect();
    let any = |st: &str| open.iter().any(|k| k.status == st);
    match t.status.as_str() {
        "cancelled" => Lane::Dropped,
        "done" => Lane::Done,
        _ if asks_owner(t) || open.iter().any(|k| asks_owner(k)) => Lane::NeedsYou,
        "failed" | "blocked" => Lane::Stuck,
        _ if any("running") => Lane::InFlight,
        _ if any("failed") || any("blocked") => Lane::Stuck,
        "running" => Lane::InFlight,
        _ if t.parent.is_none() && open.is_empty() => Lane::Planned,
        _ => Lane::Queued,
    }
}
