#![allow(dead_code, unused_imports)]
pub use kfk_store::*;
pub use kfk_usage::{Win, Wins, LOGIN};
pub use std::collections::HashMap;

pub fn temp() -> Store {
    Store::temp()
}



pub fn t(title: &str, parent: Option<i64>) -> Task {
    Task { title: title.into(), parent, spec: "s".into(), dod: "d".into(), chk: "true".into(), class: "impl".into(), owner: "o1".into(), ..Default::default() }
}
pub fn win(u: f64) -> Wins {
    [("seven_day".to_string(), Win { util: Some(u), reset: None, status: None })].into_iter().collect()
}
