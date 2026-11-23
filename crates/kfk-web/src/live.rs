//! the real probe: tmux, mizan's pid file, PostgreSQL
use crate::probe::Probe;
use anyhow::Result;
use kfk_store::{Seat, Store};

pub struct Live;

impl Probe for Live {
    fn fleet_up(&self, s: &Store) -> bool {
        kfk_mizan::session_up(s)
    }
    fn mizan(&self, s: &Store) -> bool {
        kfk_mizan::mizan_pid(&s.kh).is_some()
    }
    fn node_state(&self, s: &Store, seat: &Seat) -> String {
        kfk_mizan::node_state(s, seat).to_string()
    }
    fn pane(&self, s: &Store, seat: &str) -> String {
        kfk_mizan::capture(s, seat)
    }
    fn archived(&self, s: &Store, path: &str) -> Result<Vec<String>> {
        kfk_logs::read_lines(s, path)
    }
}
