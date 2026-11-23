//! the environment an agent or node gets

pub fn claude_bin() -> String {
    std::env::var("KFK_CLAUDE").ok().filter(|s| !s.is_empty()).unwrap_or_else(|| "claude".into())
}

/// the environment a claude child gets: nothing inherited from the calling session, the token if there is one
pub fn clean_env(tok: Option<&str>, extra: &[(&str, String)]) -> Vec<(String, String)> {
    let drop = ["CLAUDECODE", "ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN", "KFK_SEAT", "KFK_SLOT", "KFK_ROLE", "KFK_ACCOUNT", "KFK_TASK"];
    let mut e: Vec<(String, String)> = std::env::vars().filter(|(k, _)| !(k.starts_with("CLAUDE_CODE_") || drop.contains(&k.as_str()))).collect();
    if let Some(t) = tok {
        e.push(("CLAUDE_CODE_OAUTH_TOKEN".into(), t.into()));
    }
    e.extend(extra.iter().map(|(k, v)| (k.to_string(), v.clone())));
    e
}
