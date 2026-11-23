use kfk_web::paging::*;
use kfk_web::transcript::*;

fn one(line: &str) -> Entry {
    let v = entries(line);
    assert_eq!(v.len(), 1, "{v:?}");
    v[0].clone()
}

#[test]
fn lines_become_readable_entries() {
    let e = one(r#"{"type":"user","timestamp":"2026-11-22T10:11:12.000Z","message":{"content":"do it"}}"#);
    assert_eq!((e.kind, e.body.as_str(), clock(&e.ts)), (Kind::User, "do it", "10:11:12"));
    assert_eq!(one(r#"{"type":"assistant","message":{"content":"plain"}}"#).kind, Kind::Assistant);
    let v = entries(r#"{"type":"assistant","message":{"content":[{"type":"thinking","thinking":"hm"},{"type":"thinking","thinking":" "},{"type":"text","text":"  "},{"type":"text","text":"Done."},{"type":"tool_use","name":"Bash","input":{"command":"ls -la"}},{"type":"tool_use","name":"Read","input":{"file_path":"/a/b.rs"}},{"type":"tool_use","name":"Grep","input":{"pattern":"x"}},{"type":"image"}]}}"#);
    assert_eq!(v.iter().map(|e| (e.kind, e.title.as_str())).collect::<Vec<_>>(), [(Kind::Thinking, ""), (Kind::Assistant, ""), (Kind::Tool, "Bash"), (Kind::Tool, "Read"), (Kind::Tool, "Grep"), (Kind::Other, "image")]);
    assert_eq!((v[2].body.as_str(), v[3].body.as_str()), ("ls -la", "/a/b.rs"));
    assert!(v[4].body.contains("\"pattern\": \"x\""));
    let v = entries(r#"{"type":"user","message":{"content":[{"type":"text","text":"hi"},{"type":"tool_result","content":"ok","is_error":false},{"type":"tool_result","content":[{"type":"text","text":"boom"},{"type":"image"}],"is_error":true},{"type":"tool_result"},{"type":"tool_result","content":7}]}}"#);
    assert_eq!(v.iter().map(|e| (e.kind, e.body.as_str(), e.error)).collect::<Vec<_>>(), [(Kind::User, "hi", false), (Kind::ToolResult, "ok", false), (Kind::ToolResult, "boom\n[image]", true), (Kind::ToolResult, "", false), (Kind::ToolResult, "7", false)]);
    assert!(entries(r#"{"type":"user","message":{}}"#).is_empty());
    let e = one(r#"{"type":"system","subtype":"init","content":"started"}"#);
    assert_eq!((e.kind, e.title.as_str(), e.body.as_str()), (Kind::System, "init", "started"));
    assert_eq!(one(r#"{"type":"system"}"#).title, "");
    let e = one(r#"{"type":"result","is_error":true,"result":"limit"}"#);
    assert_eq!((e.kind, e.body.as_str(), e.error), (Kind::Result, "limit", true));
    assert!(!one(r#"{"type":"result"}"#).error);
    assert_eq!(one(r#"{"type":"rate_limit_event"}"#).title, "rate_limit_event");
    assert_eq!(one(r#"{"no":"type"}"#).title, "?");
    let e = one("plain text");
    assert_eq!((e.kind, e.title.as_str(), e.body.as_str()), (Kind::Other, "not JSON", "plain text"));
    assert_eq!(clock(""), "");
}

#[test]
fn long_bodies_are_clipped_on_a_character_boundary() {
    assert_eq!(clip("short"), "short");
    let long = "é".repeat(MAX_BODY); // two bytes each
    let c = clip(&long);
    assert!(c.starts_with("éé") && c.ends_with("more bytes not shown)") && c.len() < long.len());
}

#[test]
fn kinds_have_labels_and_some_fold() {
    let all = [Kind::User, Kind::Assistant, Kind::Thinking, Kind::Tool, Kind::ToolResult, Kind::System, Kind::Result, Kind::Other];
    assert_eq!(all.iter().filter(|k| k.folded()).count(), 4);
    assert!(all.iter().all(|k| !k.label().is_empty() && !k.slug().is_empty()));
    assert!(!Kind::Assistant.folded() && !Kind::Tool.folded());
}

#[test]
fn the_newest_page_is_the_default() {
    assert_eq!(paging(0, 100, None), Paging { page: 1, pages: 1, from: 0, to: 0, total: 0 });
    assert_eq!(paging(250, 100, None), Paging { page: 3, pages: 3, from: 200, to: 250, total: 250 });
    assert_eq!(paging(250, 100, Some(1)), Paging { page: 1, pages: 3, from: 0, to: 100, total: 250 });
    assert_eq!(paging(250, 100, Some(99)).page, 3);
    assert_eq!(paging(250, 100, Some(0)).page, 1);
    assert_eq!(paging(5, 0, None), Paging { page: 5, pages: 5, from: 4, to: 5, total: 5 });
}
