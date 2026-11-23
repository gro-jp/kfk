//! which files are archived
use crate::prelude::*;

/// every *.jsonl under `root`, recursively
pub fn find(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(root) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            find(&p, out);
        } else if p.extension().map(|x| x == "jsonl").unwrap_or(false) {
            out.push(p);
        }
    }
}

/// where Claude keeps its session transcripts
pub fn projects_root(s: &Store) -> PathBuf {
    let proj = s.cfg("claude_projects");
    if proj.is_empty() {
        PathBuf::from(format!("{}/.claude/projects", std::env::var("HOME").unwrap_or_default()))
    } else {
        PathBuf::from(proj)
    }
}

pub fn sources(s: &Store) -> Vec<(String, PathBuf)> {
    let mut v = vec![];
    for (name, root) in [("session", projects_root(s)), ("agent", PathBuf::from(format!("{}/runs", s.kh)))] {
        let mut f = vec![];
        find(&root, &mut f);
        f.sort();
        v.extend(f.into_iter().map(|p| (name.to_string(), p)));
    }
    v
}
