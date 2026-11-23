//! token spend of the nodes, read from their transcripts as they grow (no LLM, no calls)
use crate::prelude::*;
use std::io::{Read, Seek, SeekFrom};

/// the transcript of a session: `<projects>/<any project>/<sid>.jsonl`
pub fn transcript(projects: &std::path::Path, sid: &str) -> Option<String> {
    let rd = std::fs::read_dir(projects).ok()?;
    rd.flatten().map(|e| e.path().join(format!("{sid}.jsonl"))).find(|p| p.is_file()).map(|p| p.to_string_lossy().into_owned())
}

/// the complete lines appended to `path` after `done` bytes
pub fn appended(path: &str, done: i64) -> Result<String> {
    let mut f = std::fs::File::open(path)?;
    if f.metadata()?.len() as i64 <= done {
        return Ok(String::new());
    }
    f.seek(SeekFrom::Start(done as u64))?;
    let mut buf = vec![];
    f.read_to_end(&mut buf)?;
    let end = buf.iter().rposition(|b| *b == b'\n').map(|i| i + 1).unwrap_or(0);
    buf.truncate(end);
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// fold what every open session wrote since the last round; a session whose node moved on is closed
pub fn fold_sessions(s: &Store) -> Result<i64> {
    let (projects, mut n) = (kfk_logs::projects_root(s), 0);
    for seat in s.seats()?.into_iter().filter(|x| x.state == "up") {
        if let Some(sid) = seat.sid.as_deref().filter(|sid| !matches!(s.session(sid), Ok(Some(x)) if x.ended.is_none())) {
            s.session_open(sid, &seat.seat, &seat.role, seat.slot.unwrap_or(LOGIN), &seat.model, &seat.effort)?;
        }
    }
    for se in s.sessions(true)? {
        let path = if se.path.is_empty() { transcript(&projects, &se.sid) } else { Some(se.path.clone()) };
        if let Some(path) = path {
            let text = appended(&path, se.bytes_done).unwrap_or_default();
            if !text.is_empty() || se.path.is_empty() {
                let mut last: kfk_spend::Last = serde_json::from_str(&se.last).unwrap_or_default();
                let b: Vec<_> = kfk_spend::fold(&text, &mut last).into_iter().map(|b| (b.hour, b.u, b.turns)).collect();
                s.session_progress(&se.sid, &path, se.bytes_done + text.len() as i64, &serde_json::to_string(&last)?, &b)?;
                n += b.len() as i64;
            }
        }
        let live = s.seat(&se.seat)?.map(|x| x.state == "up" && x.sid.as_deref() == Some(se.sid.as_str())).unwrap_or(false);
        if !live {
            s.session_end(&se.sid)?;
        }
    }
    Ok(n)
}
