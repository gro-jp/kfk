//! what every command needs: the store and the token vault
use anyhow::Result;
use kfk_store::Store;
use kfk_token::Vault;

pub struct Ctx {
    pub s: Store,
    pub v: Box<dyn Vault>,
}

/// the data home: $KFK_HOME, else what the installer recorded, else ~/.kfk
pub fn home() -> String {
    let user = std::env::var("HOME").unwrap_or_default();
    let pointer = std::fs::read_to_string(kfk_util::home_pointer(std::env::var("XDG_CONFIG_HOME").ok().as_deref(), &user)).ok();
    kfk_util::resolve_home(std::env::var("KFK_HOME").ok().as_deref(), pointer.as_deref(), &user)
}

impl Ctx {
    pub fn open() -> Result<Ctx> {
        Ok(Ctx { s: Store::open(&home())?, v: kfk_token::vault() })
    }
}

pub fn slot_name(slot: Option<i64>) -> String {
    match slot {
        None | Some(kfk_usage::LOGIN) => "login".into(),
        Some(n) => n.to_string(),
    }
}
