//! minting an account token: browser sign-in on a wide pty, read back, verified, then stored
use crate::ctx::Ctx;
use anyhow::{bail, Result};
use kfk_engine::claude_bin;
use kfk_token::{from_log, label_ok, shape_ok, LEN};
use kfk_util::{now, parse_slots, stamp};
use std::io::{BufRead, Write};
use std::process::Command;

fn ask(q: &str) -> Result<String> {
    print!("{q}");
    std::io::stdout().flush()?;
    let mut a = String::new();
    std::io::stdin().lock().read_line(&mut a)?;
    Ok(a.trim().to_string())
}

pub fn auth(c: &Ctx, slots: &str, paste: bool, log: Option<String>, label: Option<String>) -> Result<i32> {
    let list = parse_slots(slots)?;
    let (tok, kept) = if paste {
        (ask("paste the token: ")?, None)
    } else {
        let path = match &log {
            Some(p) => p.clone(),
            None => {
                let p = format!("{}/mint-{}.log", c.s.kh, now() as i64);
                println!("kfk: a browser sign-in follows; sign in as the account for slot(s) {slots}. Do not resize this window.");
                // a pty forced to 200 columns: the CLI lays the token out for the terminal width, and a narrow one cuts it
                Command::new("script").args(["-q", &p, "/bin/bash", "-c", "stty cols 200 rows 60 2>/dev/null; exec \"$@\"", "kfk", &claude_bin(), "setup-token"]).status()?;
                let _ = Command::new("chmod").args(["600", &p]).status();
                p
            }
        };
        let raw = String::from_utf8_lossy(&std::fs::read(&path)?).into_owned();
        match from_log(&raw) {
            Some(t) => (t, Some(path)),
            None => bail!("no token found in the transcript {path}; sign in again: kfk auth {slots}"),
        }
    };
    let retry = kept.as_ref().map(|p| format!(" The transcript is kept: kfk auth {slots} --from-log {p}")).unwrap_or_default();
    if !shape_ok(&tok) {
        bail!("that is not a whole setup-token ({} characters, expected {LEN}); nothing stored.{retry}", tok.len());
    }
    let (rc, o) = kfk_engine::ping(Some(&tok));
    if rc == 1 {
        bail!("the server rejected this token; nothing stored.{retry}");
    }
    if rc == 2 {
        println!("kfk: note - the verification call failed for a reason other than auth (offline?); storing anyway");
    }
    let label = match label {
        Some(l) => l,
        None => ask("account label (its email): ")?,
    };
    if !label_ok(&label) {
        bail!("the label {label:?} is not an email-shaped word; nothing stored.{retry}");
    }
    for slot in &list {
        c.v.put(*slot, &tok, &label)?;
        c.s.put_account(*slot, &label, &stamp(now(), "%Y-%m-%d"))?;
        c.s.record_windows(*slot, &o.wins, "haiku")?;
    }
    c.s.event(None, "auth", &format!("slots {slots} = {label}"));
    if let (Some(p), None) = (&kept, &log) {
        let _ = std::fs::remove_file(p);
    }
    println!("slot(s) {slots} = {label}, {}", if rc == 0 { "verified ok" } else { "NOT verified" });
    Ok(0)
}
