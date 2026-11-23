//! what the pages need from outside the store: tmux, the daemon, the log archive
use anyhow::Result;
use kfk_store::{Seat, Store};

pub trait Probe: Send + Sync {
    fn fleet_up(&self, s: &Store) -> bool;
    fn mizan(&self, s: &Store) -> bool;
    /// up | dead | down | stopped
    fn node_state(&self, s: &Store, seat: &Seat) -> String;
    /// the visible text of a node's pane
    fn pane(&self, s: &Store, seat: &str) -> String;
    /// the archived lines of a log file that is no longer on disk
    fn archived(&self, s: &Store, path: &str) -> Result<Vec<String>>;
}
