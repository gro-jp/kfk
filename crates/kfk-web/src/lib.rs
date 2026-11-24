//! the web interface: read-only pages on the fleet and a live terminal on each node (Axum, Maud, HTMX)
pub mod lane;
pub mod live;
pub mod model;
pub mod page;
pub mod paging;
pub mod probe;
pub mod routes;
pub mod serve;
pub mod shell;
pub mod source;
pub mod transcript;
pub mod ws;

pub use routes::{router, App};
pub use serve::{serve, web_pid, web_start, web_stop};
