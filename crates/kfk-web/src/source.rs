//! the lines of a transcript: from disk while the file is there, from the archive after that
use crate::paging::{paging, Paging};
use crate::probe::Probe;
use crate::transcript::{entries, Entry, Kind};
use anyhow::Result;
use kfk_store::Store;

pub const PER_PAGE: usize = 150;

pub struct Log {
    pub entries: Vec<Entry>,
    pub paging: Paging,
    /// "disk", "archive", or why there is nothing
    pub origin: String,
    pub all: bool,
}

pub fn lines(s: &Store, p: &dyn Probe, path: &str) -> Result<(Vec<String>, &'static str)> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok((text.lines().map(String::from).collect(), "disk")),
        Err(_) => Ok((p.archived(s, path)?, "archive")),
    }
}

/// one page of a transcript; `all` keeps the bookkeeping lines a reader rarely wants
pub fn log(s: &Store, p: &dyn Probe, path: &str, page: Option<usize>, all: bool) -> Log {
    let (lines, origin) = match lines(s, p, path) {
        Ok((l, o)) => (l, o.to_string()),
        Err(e) => (vec![], e.to_string()),
    };
    let paging = paging(lines.len(), PER_PAGE, page);
    let entries = lines[paging.from..paging.to].iter().flat_map(|l| entries(l)).filter(|e| all || e.kind != Kind::Other).collect();
    Log { entries, paging, origin, all }
}
