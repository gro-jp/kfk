mod common;
use common::*;

#[test]
fn workdir_shapes() {
    let s = Store::temp();
    let plain = task(&s, "no repo", "x", "true", "");
    assert_eq!(workdir(&s, &plain).unwrap(), format!("{}/work/T{}", s.kh, plain.id));
    let notgit = format!("{}/plain", s.kh);
    fs::create_dir_all(&notgit).unwrap();
    let p = task(&s, "dir, not a repo", "x", "true", &notgit);
    assert_eq!(workdir(&s, &p).unwrap(), notgit);
    let r = repo(&s);
    let g = task(&s, "repo", "x", "true", &r);
    let wt = workdir(&s, &g).unwrap();
    assert_eq!(wt, format!("{}/wt/T{}", s.kh, g.id));
    assert_eq!(workdir(&s, &g).unwrap(), wt); // kept across attempts
    fs::remove_dir_all(&wt).unwrap();
    assert!(git(&r, &["worktree", "prune"]).0);
    assert_eq!(workdir(&s, &g).unwrap(), wt); // the branch exists already: re-attached
    let bad = Task { base: "no-such-ref".into(), ..task(&s, "bad base", "x", "true", &r) };
    assert!(workdir(&s, &bad).unwrap_err().to_string().contains("git worktree add failed"));
    assert!(!git("/nonexistent", &["status"]).0);
    assert!(!git_with("/nonexistent/git", &r, &["status"]).0);
}
