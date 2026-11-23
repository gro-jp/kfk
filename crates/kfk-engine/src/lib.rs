//! running agents: one short-lived `claude -p` per attempt, in the task's own worktree, judged by the task's check
mod attempt;
mod check;
mod detach;
mod env;
mod exec;
mod ping;
mod proc;
mod prompt;
mod runner;
mod workdir;

pub use attempt::*;
pub use check::*;
pub use detach::*;
pub use env::*;
pub use exec::*;
pub use ping::*;
pub use proc::*;
pub use prompt::*;
pub use runner::*;
pub use workdir::*;

#[allow(unused_imports)]
mod prelude {
    pub use crate::*;
    pub use kfk_store::{Gate, Store, Task};
    pub use kfk_stream::Parsed;
    pub use kfk_token::Vault;
    pub use kfk_usage::{Win, Wins};
    pub use kfk_util::{now, tail_lines, until};
    pub use anyhow::{anyhow, Result};
    pub use std::fs::{self, File};
    pub use std::os::unix::process::CommandExt;
    pub use std::process::{Child, Command, Stdio};
    pub use std::time::{Duration, Instant};
    pub use kfk_route as route;
    pub use kfk_stream as stream;
    pub use kfk_usage::LOGIN;
}
