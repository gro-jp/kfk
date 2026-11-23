#![allow(dead_code, unused_imports)]
pub use kfk_engine::*;
pub use kfk_store::{Store, Task};
pub use kfk_token::{FileVault, Vault};
pub use kfk_usage::{Win, Wins, LOGIN};
pub use kfk_util::now;
pub use std::fs;
pub use std::os::unix::fs::PermissionsExt;
pub use std::sync::Mutex;
pub static ENV: Mutex<()> = Mutex::new(()); // KFK_CLAUDE is process-wide

/// a fake `claude`: writes out.txt as the prompt asks, commits, prints a stream
pub fn fake(dir: &str, body: &str) -> String {
    let p = format!("{dir}/fake-claude-{}", uuid::Uuid::new_v4());
    fs::write(&p, format!("#!/bin/bash\np=$(cat)\n{body}\n")).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
    p
}
pub const GOOD: &str = r#"if echo "$p" | grep -q FLAKY && ! echo "$p" | grep -q 'previous attempt'; then echo wrong > out.txt; else echo good > out.txt; fi
git add -A >/dev/null 2>&1; git -c user.email=t@t -c user.name=t commit -qm agent >/dev/null 2>&1
echo '{"type":"rate_limit_event","rate_limit_info":{"status":"allowed","rateLimitType":"seven_day","utilization":0.2,"resetsAt":1999999999}}'
echo '{"type":"result","is_error":false,"num_turns":2,"total_cost_usd":0.01,"result":"did it","usage":{"input_tokens":10,"cache_creation_input_tokens":100,"cache_read_input_tokens":1000,"output_tokens":50}}'"#;
pub const LIMITED: &str = r#"if [ "$CLAUDE_CODE_OAUTH_TOKEN" = tok0 ]; then echo '{"type":"result","is_error":true,"result":"Claude usage limit reached"}'; exit 1; fi
echo good > out.txt; echo '{"type":"result","is_error":false,"result":"ok","usage":{}}'"#;

pub fn task(s: &Store, title: &str, spec: &str, chk: &str, repo: &str) -> Task {
    let id = s.add_task(&Task { title: title.into(), spec: spec.into(), dod: "d".into(), chk: chk.into(), class: "impl".into(), repo: repo.into(), ..Default::default() }).unwrap();
    s.task(id).unwrap()
}
pub fn vault(s: &Store) -> FileVault {
    FileVault(format!("{}/tok.json", s.kh))
}
pub fn repo(s: &Store) -> String {
    let r = format!("{}/repo", s.kh);
    fs::create_dir_all(&r).unwrap();
    assert!(git(&r, &["init", "-q"]).0);
    assert!(git(&r, &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-q", "--allow-empty", "-m", "init"]).0);
    r
}
