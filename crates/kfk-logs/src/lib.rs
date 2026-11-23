//! the log archive: every Claude session transcript and every agent stream, copied into PostgreSQL and never
//! deleted. Incremental by byte offset.
mod find;
mod parse;
mod pg;
mod read;
mod status;
mod sync;

pub use find::*;
pub use parse::*;
pub use pg::*;
pub use read::*;
pub use status::*;
pub use sync::*;

#[allow(unused_imports)]
mod prelude {
    pub use crate::*;
    pub use kfk_store::Store;
    pub use anyhow::{anyhow, bail, Result};
    pub use postgres::{Client, NoTls};
    pub use serde_json::Value;
    pub use std::io::{Read, Seek, SeekFrom};
    pub use std::path::{Path, PathBuf};
    pub use std::process::Command;
}
