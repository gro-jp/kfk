//! one line of a Claude transcript (a session file or an agent's stream) as entries a person can read
use serde_json::Value;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    User,
    Assistant,
    Thinking,
    Tool,
    ToolResult,
    System,
    Result,
    Other,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::User => "prompt",
            Kind::Assistant => "claude",
            Kind::Thinking => "thinking",
            Kind::Tool => "tool call",
            Kind::ToolResult => "tool result",
            Kind::System => "system",
            Kind::Result => "result",
            Kind::Other => "other",
        }
    }
    pub fn slug(self) -> &'static str {
        match self {
            Kind::User => "user",
            Kind::Assistant => "assistant",
            Kind::Thinking => "thinking",
            Kind::Tool => "tool",
            Kind::ToolResult => "toolresult",
            Kind::System => "system",
            Kind::Result => "result",
            Kind::Other => "other",
        }
    }
    /// shown folded: long, and rarely what the reader came for
    pub fn folded(self) -> bool {
        matches!(self, Kind::Thinking | Kind::ToolResult | Kind::System | Kind::Other)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Entry {
    pub ts: String,
    pub kind: Kind,
    pub title: String,
    pub body: String,
    pub error: bool,
}

pub const MAX_BODY: usize = 20_000;

/// cut a long body at a character boundary and say so
pub fn clip(s: &str) -> String {
    if s.len() <= MAX_BODY {
        return s.to_string();
    }
    let cut = (0..=MAX_BODY).rev().find(|i| s.is_char_boundary(*i)).unwrap_or(0);
    format!("{}\n… ({} more bytes not shown)", &s[..cut], s.len() - cut)
}

fn text_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Array(a) => a.iter().map(|b| b.get("text").and_then(Value::as_str).map(String::from).unwrap_or_else(|| format!("[{}]", b.get("type").and_then(Value::as_str).unwrap_or("block")))).collect::<Vec<_>>().join("\n"),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// what a tool was asked to do, in the shortest honest form
fn tool_input(name: &str, input: &Value) -> String {
    let field = |k: &str| input.get(k).and_then(Value::as_str);
    match (name, field("command"), field("file_path")) {
        ("Bash", Some(cmd), _) => cmd.to_string(),
        (_, _, Some(path)) if matches!(name, "Read" | "Write" | "Edit") => path.to_string(),
        _ => serde_json::to_string_pretty(input).unwrap_or_default(),
    }
}

fn block(ts: &str, role: &str, b: &Value) -> Option<Entry> {
    let e = |kind, title: &str, body: String, error| Some(Entry { ts: ts.to_string(), kind, title: title.to_string(), body: clip(&body), error });
    let s = |k: &str| b.get(k).and_then(Value::as_str).unwrap_or("");
    match s("type") {
        "text" if s("text").trim().is_empty() => None,
        "text" => e(if role == "user" { Kind::User } else { Kind::Assistant }, "", s("text").to_string(), false),
        "thinking" if s("thinking").trim().is_empty() => None,
        "thinking" => e(Kind::Thinking, "", s("thinking").to_string(), false),
        "tool_use" => e(Kind::Tool, s("name"), tool_input(s("name"), b.get("input").unwrap_or(&Value::Null)), false),
        "tool_result" => e(Kind::ToolResult, "", text_of(b.get("content").unwrap_or(&Value::Null)), b.get("is_error").and_then(Value::as_bool).unwrap_or(false)),
        other => e(Kind::Other, other, String::new(), false),
    }
}

/// the entries of one line; a line that is not JSON is shown as it is
pub fn entries(line: &str) -> Vec<Entry> {
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return vec![Entry { ts: String::new(), kind: Kind::Other, title: "not JSON".into(), body: clip(line), error: false }];
    };
    let ts = v.get("timestamp").and_then(Value::as_str).unwrap_or("").to_string();
    let one = |kind, title: &str, body: String, error| vec![Entry { ts: ts.clone(), kind, title: title.to_string(), body: clip(&body), error }];
    let ty = v.get("type").and_then(Value::as_str).unwrap_or("?");
    match ty {
        "user" | "assistant" => match v.pointer("/message/content") {
            Some(Value::String(s)) => one(if ty == "user" { Kind::User } else { Kind::Assistant }, "", s.clone(), false),
            Some(Value::Array(a)) => a.iter().filter_map(|b| block(&ts, ty, b)).collect(),
            _ => vec![],
        },
        "system" => one(Kind::System, v.get("subtype").and_then(Value::as_str).unwrap_or(""), text_of(v.get("content").unwrap_or(&Value::Null)), false),
        "result" => one(Kind::Result, "", text_of(v.get("result").unwrap_or(&Value::Null)), v.get("is_error").and_then(Value::as_bool).unwrap_or(false)),
        other => one(Kind::Other, other, String::new(), false),
    }
}

/// HH:MM:SS of an ISO timestamp, or nothing
pub fn clock(ts: &str) -> &str {
    ts.get(11..19).unwrap_or("")
}
