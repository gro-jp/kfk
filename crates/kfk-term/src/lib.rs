//! a terminal on a node: who may open one, what the browser sends, and the tmux client behind it
pub mod attach;
pub mod guard;
pub mod msg;
pub mod pty;

pub use attach::{attach_args, seat_ok};
pub use guard::{admit, usable, Ask, Deny};
pub use msg::{resize, size};
pub use pty::Pty;
