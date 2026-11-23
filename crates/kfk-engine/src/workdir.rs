//! where an agent works: a git worktree per task
use crate::prelude::*;

pub fn git(repo: &str, args: &[&str]) -> (bool, String) {
    git_with("git", repo, args)
}

/// (succeeded, what it printed) of `<bin> -C <repo> <args>`
pub fn git_with(bin: &str, repo: &str, args: &[&str]) -> (bool, String) {
    match Command::new(bin).arg("-C").arg(repo).args(args).output() {
        Ok(o) => (o.status.success(), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)).trim().to_string()),
        Err(e) => (false, e.to_string()),
    }
}

/// a worktree of the task's repo on branch kfk/T<id> (kept across attempts), or a plain directory
pub fn workdir(s: &Store, t: &Task) -> Result<String> {
    if !t.repo.is_empty() && std::path::Path::new(&t.repo).is_dir() {
        let (ok, top) = git(&t.repo, &["rev-parse", "--show-toplevel"]);
        if !ok {
            return Ok(t.repo.clone());
        }
        let (wt, br) = (format!("{}/wt/T{}", s.kh, t.id), format!("kfk/T{}", t.id));
        if !std::path::Path::new(&wt).is_dir() {
            fs::create_dir_all(format!("{}/wt", s.kh))?;
            let base = if t.base.is_empty() { "HEAD" } else { &t.base };
            if !git(&top, &["worktree", "add", "-b", &br, &wt, base]).0 {
                let (ok, err) = git(&top, &["worktree", "add", &wt, &br]);
                if !ok {
                    return Err(anyhow!("git worktree add failed: {}", tail_lines(&err, 3, 300)));
                }
            }
        }
        return Ok(wt);
    }
    let d = format!("{}/work/T{}", s.kh, t.id);
    fs::create_dir_all(&d)?;
    Ok(d)
}
