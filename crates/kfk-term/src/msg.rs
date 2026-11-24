//! what the browser sends besides keystrokes: the size of its terminal

pub const DEFAULT: (u16, u16) = (120, 32);

/// columns and rows a terminal can sensibly have
pub fn size(cols: Option<u64>, rows: Option<u64>) -> (u16, u16) {
    (cols.map(|c| c.clamp(20, 500) as u16).unwrap_or(DEFAULT.0), rows.map(|r| r.clamp(5, 300) as u16).unwrap_or(DEFAULT.1))
}

/// a text frame `{"cols":120,"rows":32}`; anything else is not a resize
pub fn resize(text: &str) -> Option<(u16, u16)> {
    let v: serde_json::Value = serde_json::from_str(text).ok()?;
    Some(size(Some(v.get("cols")?.as_u64()?), Some(v.get("rows")?.as_u64()?)))
}
