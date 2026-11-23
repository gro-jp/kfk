//! the web interface: a read-only view of the fleet (Axum, Maud, HTMX)
pub mod lane;
pub mod live;
pub mod model;
pub mod page;
pub mod paging;
pub mod probe;
pub mod routes;
pub mod serve;
pub mod source;
pub mod transcript;

pub use routes::{router, App};
pub use serve::{serve, web_pid, web_start, web_stop};
