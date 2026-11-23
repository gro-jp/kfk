//! reading a file back out of the archive
use crate::prelude::*;

/// every archived line of `path`, in order; an error when the archive is not set up or does not hold the file
pub fn read_lines(s: &Store, path: &str) -> Result<Vec<String>> {
    if !initialised(s) || !running(s) {
        bail!("{path} is not on disk and the log archive is not running (kfk logs sync)");
    }
    let rows = connect(s, "kfk")?.query("select coalesce(l.data::text, l.raw, '') from log_lines l join log_files f on f.id=l.file_id where f.path=$1 order by l.off", &[&path])?;
    if rows.is_empty() {
        bail!("{path} is neither on disk nor in the log archive");
    }
    Ok(rows.iter().map(|r| r.get::<_, String>(0)).collect())
}
