use crate::ctx::Ctx;
use anyhow::Result;
use kfk_store::DEFAULTS;
use kfk_util::table;

pub fn config(c: &Ctx, key: Option<String>, value: Option<String>) -> Result<i32> {
    match (key, value) {
        (None, _) => {
            let rows: Vec<Vec<String>> = DEFAULTS.iter().map(|(k, d)| { let v = c.s.cfg(k); let note = if v == *d { String::new() } else { format!("(default {d})") }; vec![k.to_string(), v, note] }).collect();
            println!("{}", table(&["key", "value", ""], &rows));
        }
        (Some(k), None) => println!("{}", c.s.cfg(&k)),
        (Some(k), Some(v)) => {
            c.s.set_cfg(&k, &v)?;
            println!("{k} = {v}");
        }
    }
    Ok(0)
}
