mod common;
use common::*;
use kfk_web::page::parts::*;
use kfk_web::page::log_part;
use kfk_web::source::log;

#[test]
fn small_pieces_say_what_they_mean() {
    assert_eq!((dur(59.0), dur(125.0), dur(3725.0), dur(-5.0)), ("0m 59s".to_string(), "2m 05s".to_string(), "1h 02m".to_string(), "0m 00s".to_string()));
    assert_eq!((dash(" "), dash("x")), ("–", "x"));
    for (state, class) in [("up", "st-ok"), ("running", "st-go"), ("failed", "st-bad"), ("limit", "st-warn"), ("todo", "st-wait"), ("stopped", "st-off")] {
        let h = state_badge(state).into_string();
        assert!(h.contains(class) && h.contains(state), "{h}");
    }
    assert!(meter(None, "x").into_string().contains("unknown"));
    for (pct, class, shown) in [(12.4, "m-ok", "12%"), (85.0, "m-warn", "85%"), (99.0, "m-bad", "99%"), (250.0, "m-bad", "100%")] {
        let h = meter(Some(pct), "usage").into_string();
        assert!(h.contains(class) && h.contains(shown) && h.contains("aria-label=\"usage\""), "{h}");
    }
    assert!(when(None).into_string().contains("–") && when(Some(0.0)).into_string().contains("–"));
    assert!(when(Some(kfk_util::now() - 120.0)).into_string().contains("2m ago"));
    assert!(clock(None).into_string().contains("–") && clock(Some(86400.0 * 400.0)).into_string().contains("<time datetime=\"1971-"));
    assert!(tok(1500).into_string().contains("1500 weighted tokens"));
    assert_eq!(block("Spec", "  ").into_string(), "");
    assert!(block("Spec", " a<b ").into_string().contains("<pre>a&lt;b</pre>"));
    assert!(!stat("l", maud::html! { "v" }, "").into_string().contains("stat-n"));
    assert!(stat("l", maud::html! { "v" }, "n").into_string().contains("stat-n"));
    assert!(task_link(7).into_string().contains("href=\"/task/7\""));
    assert!(empty("nothing").into_string().contains("nothing"));
}

#[test]
fn a_long_transcript_is_paged_oldest_to_newest() {
    let s = kfk_store::Store::temp();
    let path = format!("{}/long.jsonl", s.kh);
    let text: String = (0..400).map(|i| format!("{{\"type\":\"user\",\"message\":{{\"content\":\"line {i}\"}}}}\n")).collect();
    std::fs::write(&path, text).unwrap();
    let p = Fake::default();
    let last = log_part(&log(&s, &p, &path, None, false), "/session/x").into_string();
    assert!(last.contains("Page 3 of 3") && last.contains("line 399") && !last.contains("line 299<") && last.contains("href=\"/session/x?page=2\"") && !last.contains("Newer"));
    let mid = log_part(&log(&s, &p, &path, Some(2), true), "/session/x").into_string();
    assert!(mid.contains("Page 2 of 3") && mid.contains("line 150") && mid.contains("href=\"/session/x?page=1&amp;all=1\"") && mid.contains("href=\"/session/x?page=3&amp;all=1\"") && mid.contains("Newest"));
    let first = log_part(&log(&s, &p, &path, Some(1), false), "/session/x").into_string();
    assert!(first.contains("line 0<") && !first.contains("Older"));
    // a folded entry says how long it is; an error is marked in words
    std::fs::write(&path, "{\"type\":\"user\",\"message\":{\"content\":[{\"type\":\"tool_result\",\"content\":\"a\\nb\",\"is_error\":true}]}}\n{\"type\":\"result\",\"result\":\"\"}\n").unwrap();
    let h = log_part(&log(&s, &p, &path, None, false), "/run/1").into_string();
    assert!(h.contains("<details>") && h.contains("2 lines") && h.contains("! error") && !h.contains("Page 1 of"));
    let none = log_part(&log(&s, &p, "/gone", None, false), "/run/1").into_string();
    assert!(none.contains("Nothing to show on this page."));
}
