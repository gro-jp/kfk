use anyhow::{bail, Result};
use chrono::TimeZone;

pub fn now() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

pub fn ktok(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1e6)
    } else if n >= 1000 {
        format!("{}k", n / 1000)
    } else {
        n.to_string()
    }
}

pub fn ago(ts: Option<f64>, now: f64) -> String {
    let Some(ts) = ts else { return "-".into() };
    let s = (now - ts).max(0.0) as i64;
    for (n, u) in [(86400, "d"), (3600, "h"), (60, "m")] {
        if s >= n {
            return format!("{}{}", s / n, u);
        }
    }
    format!("{s}s")
}

pub fn stamp(ts: f64, f: &str) -> String {
    chrono::Local.timestamp_opt(ts as i64, 0).single().map(|d| d.format(f).to_string()).unwrap_or_else(|| "-".into())
}

pub fn until(ts: Option<f64>) -> String {
    ts.map(|t| stamp(t, "%a %H:%M")).unwrap_or_else(|| "-".into())
}

/// one slot, a list (3,12) or a range (10-20)
pub fn parse_slots(s: &str) -> Result<Vec<i64>> {
    let mut out = vec![];
    for part in s.split(',') {
        let part = part.trim();
        let (a, b) = match part.split_once('-') {
            Some((a, b)) => (a, b),
            None => (part, part),
        };
        let (Ok(a), Ok(b)) = (a.parse::<i64>(), b.parse::<i64>()) else { bail!("{s:?} is not a slot, a list (3,12) or a range (10-20)") };
        if a > b || b > 99 {
            bail!("slots are 0-99, a range goes upward: {s:?}");
        }
        out.extend(a..=b);
    }
    Ok(out)
}

pub fn alive(pid: i64) -> bool {
    if pid <= 0 {
        return false;
    }
    let r = unsafe { libc::kill(pid as i32, 0) };
    r == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

/// input-token equivalents: cache write 1.25x, cache read 0.1x, output 5x
pub fn weighted(input: i64, cw: i64, cr: i64, out: i64) -> i64 {
    (input as f64 + 1.25 * cw as f64 + 0.1 * cr as f64 + 5.0 * out as f64) as i64
}

pub fn table(head: &[&str], rows: &[Vec<String>]) -> String {
    let mut w: Vec<usize> = head.iter().map(|h| h.chars().count()).collect();
    for r in rows {
        for (i, x) in r.iter().enumerate() {
            w[i] = w[i].max(x.chars().count());
        }
    }
    let line = |r: Vec<&str>| {
        let s: Vec<String> = r.iter().enumerate().map(|(i, x)| format!("{:<1$}", x, w[i])).collect();
        s.join("  ").trim_end().to_string()
    };
    let mut out = vec![line(head.to_vec())];
    out.extend(rows.iter().map(|r| line(r.iter().map(|s| s.as_str()).collect())));
    out.join("\n")
}

pub fn tail_lines(s: &str, n: usize, max: usize) -> String {
    let l: Vec<&str> = s.lines().collect();
    let t = l[l.len().saturating_sub(n)..].join("\n");
    let cut = t.len().saturating_sub(max);
    let cut = (cut..=t.len()).find(|i| t.is_char_boundary(*i)).unwrap_or(t.len());
    t[cut..].to_string()
}

/// Where kfk keeps its data: `$KFK_HOME`, else the directory named in the pointer file the installer wrote
/// (`<config dir>/kfk/home`), else `~/.kfk`. Empty values count as absent.
pub fn resolve_home(env: Option<&str>, pointer: Option<&str>, user_home: &str) -> String {
    let pick = |x: Option<&str>| x.map(str::trim).filter(|v| !v.is_empty()).map(String::from);
    pick(env).or_else(|| pick(pointer)).unwrap_or_else(|| format!("{user_home}/.kfk"))
}

/// the pointer file: `$XDG_CONFIG_HOME/kfk/home`, else `~/.config/kfk/home`
pub fn home_pointer(xdg: Option<&str>, user_home: &str) -> String {
    match xdg.map(str::trim).filter(|v| !v.is_empty()) {
        Some(x) => format!("{x}/kfk/home"),
        None => format!("{user_home}/.config/kfk/home"),
    }
}
