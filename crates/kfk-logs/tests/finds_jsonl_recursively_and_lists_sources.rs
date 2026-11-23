use kfk_logs::*;
use std::path::Path;

#[test]
fn finds_jsonl_recursively_and_lists_sources() {
    let s = kfk_store::Store::temp();
    let root = format!("{}/proj", s.kh);
    std::fs::create_dir_all(format!("{root}/a/b")).unwrap();
    std::fs::create_dir_all(format!("{}/runs/1", s.kh)).unwrap();
    for f in ["a/x.jsonl", "a/b/y.jsonl", "a/skip.txt"] {
        std::fs::write(format!("{root}/{f}"), "{}\n").unwrap();
    }
    std::fs::write(format!("{}/runs/1/stream.jsonl", s.kh), "{}\n").unwrap();
    s.c.execute("insert or replace into config(k,v) values('claude_projects',?1)", [&root]).unwrap();
    assert!(projects_root(&kfk_store::Store::temp()).ends_with(".claude/projects"));
    assert_eq!(projects_root(&s), Path::new(&root));
    let v = sources(&s);
    assert_eq!(v.iter().map(|(k, p)| (k.as_str(), p.file_name().unwrap().to_str().unwrap())).collect::<Vec<_>>(), [("session", "y.jsonl"), ("session", "x.jsonl"), ("agent", "stream.jsonl")]);
    let mut none = vec![];
    find(Path::new("/nonexistent"), &mut none);
    assert!(none.is_empty());
    assert!(!initialised(&s));
    sync_quiet(&s).unwrap();
    assert_eq!(status(&s).unwrap(), "log archive: not set up (kfk logs init)");
}
