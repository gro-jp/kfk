//! account tokens: reading one back out of a terminal transcript, and where they are kept
use anyhow::{bail, Result};
use regex::Regex;
use std::collections::BTreeMap;
use std::io::Write;
use std::process::{Command, Stdio};

pub const PREFIX: &str = "sk-ant-oat01-";
pub const LEN: usize = 108;

/// Read a setup-token out of a `script` transcript. Escape sequences become line breaks (the CLI places the next
/// sentence with cursor moves), and continuation lines are joined only while the token is short of a whole one.
pub fn from_log(raw: &str) -> Option<String> {
    let mut txt = raw.to_string();
    for re in [r"\x1b\][^\x07\x1b]*(\x07|\x1b\\)", r"\x1b\[[0-9;?<>=]*[ -/]*[@-~]", r"\x1b[()][A-Za-z0-9]|\x1b[78]|\x0f", "[\u{2500}-\u{257f}\u{2580}-\u{259f}\u{25a0}-\u{25ff}\u{2190}-\u{21ff}]"] {
        txt = Regex::new(re).unwrap().replace_all(&txt, "\n").into_owned();
    }
    let txt = txt.replace('\r', "\n");
    let hit = Regex::new(&format!("{}[A-Za-z0-9_-]+", regex::escape(PREFIX))).unwrap();
    let cont = Regex::new(r"^\n+[ \t]*([A-Za-z0-9_-]+)[ \t]*").unwrap();
    let mut cands = vec![];
    for m in hit.find_iter(&txt) {
        let (mut tok, mut pos) = (m.as_str().to_string(), m.end());
        while tok.len() < LEN {
            let Some(c) = cont.captures(&txt[pos..]) else { break };
            let end = pos + c.get(0).unwrap().end();
            if !(end == txt.len() || txt[end..].starts_with('\n')) {
                break;
            }
            tok.push_str(&c[1]);
            pos = end;
        }
        cands.push(tok);
    }
    cands.reverse(); // the CLI redraws its screen: newest first, a candidate of exactly the expected length wins
    cands.iter().find(|c| c.len() == LEN).or(cands.first()).cloned()
}

pub fn shape_ok(tok: &str) -> bool {
    tok.starts_with(PREFIX) && tok.len() >= 90 && tok.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub fn label_ok(label: &str) -> bool {
    !label.is_empty() && label.chars().all(|c| c.is_ascii_alphanumeric() || "._%+@-".contains(c))
}

pub trait Vault {
    fn get(&self, slot: i64) -> Option<String>;
    fn put(&self, slot: i64, tok: &str, label: &str) -> Result<()>;
    fn del(&self, slot: i64);
}

/// tests only (KFK_TOKEN_FILE): a JSON file in place of the Keychain
pub struct FileVault(pub String);

impl FileVault {
    fn load(&self) -> BTreeMap<String, String> {
        std::fs::read_to_string(&self.0).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }
}

impl Vault for FileVault {
    fn get(&self, slot: i64) -> Option<String> {
        if slot < 0 { None } else { self.load().get(&slot.to_string()).cloned() }
    }
    fn put(&self, slot: i64, tok: &str, _label: &str) -> Result<()> {
        let mut m = self.load();
        m.insert(slot.to_string(), tok.to_string());
        std::fs::write(&self.0, serde_json::to_string(&m)?)?;
        Ok(())
    }
    fn del(&self, slot: i64) {
        let mut m = self.load();
        m.remove(&slot.to_string());
        let _ = std::fs::write(&self.0, serde_json::to_string(&m).unwrap_or_default());
    }
}

/// the macOS Keychain: the service named here ("kfk"), account "slotN"; written through `security -i` on stdin,
/// never in an argv
pub struct Keychain(pub &'static str);

impl Vault for Keychain {
    fn get(&self, slot: i64) -> Option<String> {
        if slot < 0 {
            return None;
        }
        let o = Command::new("security").args(["find-generic-password", "-s", self.0, "-a", &format!("slot{slot}"), "-w"]).output().ok()?;
        let t = String::from_utf8_lossy(&o.stdout).trim().to_string();
        (o.status.success() && !t.is_empty()).then_some(t)
    }
    fn put(&self, slot: i64, tok: &str, label: &str) -> Result<()> {
        let mut p = Command::new("security").arg("-i").stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()?;
        writeln!(p.stdin.take().unwrap(), "add-generic-password -U -s {} -a slot{slot} -j \"{label}\" -w \"{tok}\"", self.0)?;
        p.wait()?;
        if self.get(slot).as_deref() != Some(tok) {
            bail!("slot {slot} did not read back what was written (Keychain locked?)");
        }
        Ok(())
    }
    fn del(&self, slot: i64) {
        let _ = Command::new("security").args(["delete-generic-password", "-s", self.0, "-a", &format!("slot{slot}")]).output();
    }
}

pub fn vault() -> Box<dyn Vault> {
    match std::env::var("KFK_TOKEN_FILE") {
        Ok(f) if !f.is_empty() => Box::new(FileVault(f)),
        _ => Box::new(Keychain("kfk")),
    }
}
