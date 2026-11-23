//! one small call that says whether an account works
use crate::prelude::*;

/// one small call on an account, in a throwaway config dir: (0 ok | 1 auth rejected | 2 other, what it printed)
pub fn ping(tok: Option<&str>) -> (i32, Parsed) {
    let d = std::env::temp_dir().join(format!("kfk-check-{}", uuid::Uuid::new_v4()));
    let _ = fs::create_dir_all(&d);
    let extra: Vec<(&str, String)> = if tok.is_some() { vec![("CLAUDE_CONFIG_DIR", d.to_string_lossy().into_owned())] } else { vec![] };
    let out = Command::new(claude_bin())
        .args(["-p", "Reply with the single word OK.", "--model", "haiku", "--output-format", "stream-json", "--verbose"])
        .current_dir(&d)
        .env_clear()
        .envs(clean_env(tok, &extra))
        .stdin(Stdio::null())
        .output();
    let _ = fs::remove_dir_all(&d);
    match out {
        Ok(o) => {
            let p = stream::parse(&format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)));
            let rc = if o.status.success() && p.result && !p.error { 0 } else if p.auth { 1 } else { 2 };
            (rc, p)
        }
        Err(_) => (2, Parsed::default()),
    }
}
