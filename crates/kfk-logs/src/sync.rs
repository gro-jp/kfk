//! copying what was appended since the last round
use crate::prelude::*;

#[derive(Default, Debug, PartialEq)]
pub struct Synced {
    pub files: i64,
    pub changed: i64,
    pub lines: i64,
    pub bytes: i64,
}

pub fn sync_file(c: &mut Client, source: &str, path: &Path) -> Result<(i64, i64)> {
    let p = path.to_string_lossy();
    let session = path.file_stem().map(|x| x.to_string_lossy().into_owned()).unwrap_or_default();
    let row = c.query_one("insert into log_files(path, session, source) values($1,$2,$3) on conflict(path) do update set path=excluded.path returning id, bytes_done", &[&p.as_ref(), &session, &source])?;
    let (id, done): (i64, i64) = (row.get(0), row.get(1));
    let mut f = std::fs::File::open(path)?;
    let len = f.metadata()?.len() as i64;
    if len <= done {
        return Ok((0, 0));
    }
    f.seek(SeekFrom::Start(done as u64))?;
    let mut buf = Vec::with_capacity((len - done) as usize);
    f.read_to_end(&mut buf)?;
    let (lines, used) = chunk(&buf);
    let mut tx = c.transaction()?;
    let st = tx.prepare("insert into log_lines(file_id, off, ts, type, data, raw) values($1,$2,$3::text::timestamptz,$4,$5,$6) on conflict do nothing")?;
    for (off, line) in &lines {
        let (ts, ty, json) = classify(line);
        let raw = if json.is_none() { Some(String::from_utf8_lossy(line).replace('\0', "")) } else { None };
        tx.execute(&st, &[&id, &(done + *off as i64), &ts, &ty, &json, &raw])?;
    }
    tx.execute("update log_files set bytes_done=$1, last_sync=now() where id=$2", &[&(done + used as i64), &id])?;
    tx.commit()?;
    Ok((lines.len() as i64, used as i64))
}

pub fn sync(s: &Store) -> Result<Synced> {
    start(s)?;
    let mut c = connect(s, "kfk")?;
    let mut r = Synced::default();
    for (source, path) in sources(s) {
        r.files += 1;
        let (lines, bytes) = sync_file(&mut c, &source, &path)?;
        if bytes > 0 {
            r.changed += 1;
            r.lines += lines;
            r.bytes += bytes;
        }
    }
    Ok(r)
}

/// mizan's call: nothing to do until `kfk logs init` has been run
pub fn sync_quiet(s: &Store) -> Result<()> {
    if initialised(s) {
        sync(s)?;
    }
    Ok(())
}
