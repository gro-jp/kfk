//! the pages, as functions from a view model to HTML
mod accounts;
mod history;
mod layout;
mod log;
mod node;
mod overview;
pub mod parts;
mod run;
mod task;

pub use accounts::*;
pub use history::*;
pub use layout::*;
pub use log::*;
pub use node::*;
pub use overview::*;
pub use run::*;
pub use task::*;
