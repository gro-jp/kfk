//! mizan: the zero-LLM seat layer. It starts every node on the best account, watches usage, rotates a node at its
//! limit, restarts dead nodes and stops idle orchestrators.
mod daemon;
mod dialog;
mod node;
mod node_main;
mod round;
mod slot;
mod spend;
mod statusline;
mod tmux;

pub use daemon::*;
pub use dialog::*;
pub use node::*;
pub use node_main::*;
pub use round::*;
pub use slot::*;
pub use spend::*;
pub use statusline::*;
pub use tmux::*;

#[allow(unused_imports)]
mod prelude {
    pub use crate::*;
    pub use kfk_engine::{claude_bin, clean_env, spawn_detached};
    pub use kfk_store::{Seat, Store};
    pub use kfk_token::Vault;
    pub use kfk_usage::{Win, Wins};
    pub use kfk_util::{alive, ktok, now, stamp};
    pub use anyhow::{anyhow, bail, Result};
    pub use serde_json::Value;
    pub use std::collections::HashMap;
    pub use std::process::{Command, Output};
    pub use kfk_usage as usage;
    pub use kfk_usage::LOGIN;
}
