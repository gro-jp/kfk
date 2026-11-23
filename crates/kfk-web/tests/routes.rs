mod common;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::*;
use http_body_util::BodyExt;
use kfk_web::{router, App};
use std::sync::Arc;
use tower::ServiceExt;

async fn get(app: &App, path: &str) -> (StatusCode, String) {
    let r = router(app.clone()).oneshot(Request::builder().uri(path).body(Body::empty()).unwrap()).await.unwrap();
    let status = r.status();
    (status, String::from_utf8(r.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap())
}

#[tokio::test]
async fn every_page_renders_and_drills_down() {
    let w = world();
    let app = App { kh: w.s.kh.clone(), probe: Arc::new(Fake::default()) };
    let (st, home) = get(&app, "/").await;
    assert_eq!(st, StatusCode::OK);
    for needle in ["<!DOCTYPE html>", "<html lang=\"en\">", "Skip to content", "aria-current=\"page\"", "hx-get=\"/part/overview\"", "Needs you", "accept the folder trust dialog", "needs owner: which region?", "stream in flight", "In flight", "Stuck", "Planned", "Queued", "a running child", "role=\"meter\"", "coordination share", "one@x"] {
        assert!(home.contains(needle), "overview lacks {needle:?}");
    }
    assert!(!home.contains("stream dropped")); // cancelled work is not on the board
    let (st, part) = get(&app, "/part/overview").await;
    assert!(st == StatusCode::OK && !part.contains("<html") && part.contains("stream in flight"));

    let (st, t) = get(&app, &format!("/task/T{}", w.failed_kid)).await;
    assert_eq!(st, StatusCode::OK);
    for needle in ["a failed child", "the spec", "the dod", "check said no", "last words", "a note from o1", &format!("/run/{}", w.run), &format!("/task/{}", w.flight), &format!("/task/{}", w.kid), "Breadcrumb"] {
        assert!(t.contains(needle), "task page lacks {needle:?}");
    }
    assert!(get(&app, &format!("/part/task/{}", w.flight)).await.1.contains("a running child"));

    let (st, r) = get(&app, &format!("/run/{}", w.run)).await;
    assert_eq!(st, StatusCode::OK);
    for needle in ["the prompt", "the verdict", "I did &lt;it&gt;", "make test", "tool call", "Show bookkeeping lines too", "4 lines, read from disk"] {
        assert!(r.contains(needle), "run page lacks {needle:?}");
    }
    assert!(!r.contains("rate_limit_event"));
    let (_, all) = get(&app, &format!("/run/{}?all=1&page=1", w.run)).await;
    assert!(all.contains("rate_limit_event") && all.contains("Hide bookkeeping lines"));

    let (st, n) = get(&app, "/nodes").await;
    assert!(st == StatusCode::OK && n.contains("/node/o2") && n.contains("stopped"));
    let (st, cos) = get(&app, "/node/cos").await;
    assert_eq!(st, StatusCode::OK);
    for needle in ["chief of staff", "working on T1", "/session/sid-cos", "Tokens by hour", "hx-get=\"/part/node/cos\""] {
        assert!(cos.contains(needle), "node page lacks {needle:?}");
    }
    let (_, o1) = get(&app, "/node/o1").await;
    assert!(o1.contains("orchestrator") && o1.contains("stream queued"));
    assert!(get(&app, "/node/o2").await.1.contains("The node is not up"));
    assert!(get(&app, "/part/node/o1").await.1.contains("Open work it owns"));
    let (st, se) = get(&app, "/session/sid-cos").await;
    assert!(st == StatusCode::OK && se.contains("the goal") && se.contains("prompt"));

    let (st, acc) = get(&app, "/accounts").await;
    assert!(st == StatusCode::OK && acc.contains("two@x") && acc.contains("97%") && acc.contains("limit") && acc.contains("1 agent"));
    let (st, h) = get(&app, "/history?days=7").await;
    assert!(st == StatusCode::OK && h.contains("Events, newest first") && h.contains("Agent runs by class and model") && h.contains("Node spend") && h.contains("Streams touched"));
    assert_eq!(get(&app, "/history?days=junk").await.0, StatusCode::OK);
    assert!(!h.contains("Event pages")); // one page: no pager
    assert_eq!(get(&app, "/history?page=junk").await.0, StatusCode::OK);

    for (path, status) in [("/task/9999", 404), ("/task/abc", 404), ("/run/9999", 404), ("/node/o9", 404), ("/session/none", 404), ("/nothing", 404)] {
        let (st, body) = get(&app, path).await;
        assert_eq!(st.as_u16(), status, "{path}");
        assert!(body.contains("Not found") && body.contains("Back to the overview"), "{path}");
    }
    assert_eq!(get(&app, "/healthz").await.1, "ok");
    assert!(get(&app, "/assets/app.css").await.1.contains("prefers-color-scheme"));
    assert!(get(&app, "/assets/htmx.min.js").await.1.contains("htmx"));
}

#[tokio::test]
async fn an_empty_home_explains_itself_and_a_broken_one_is_an_error_page() {
    let s = kfk_store::Store::temp();
    let app = App { kh: s.kh.clone(), probe: Arc::new(Fake { up: false, ..Default::default() }) };
    let (st, home) = get(&app, "/").await;
    assert_eq!(st, StatusCode::OK);
    for needle in ["Nothing is waiting for you.", "No node has been started yet", "No agent is running.", "No stream yet. Give cos a goal."] {
        assert!(home.contains(needle), "{needle:?}");
    }
    assert!(get(&app, "/nodes").await.1.contains("No node has been started yet."));
    assert!(get(&app, "/accounts").await.1.contains("The pool is empty"));
    assert!(get(&app, "/history").await.1.contains("History"));
    let broken = App { kh: "/dev/null/nope".into(), probe: Arc::new(Fake::default()) };
    let (st, body) = get(&broken, "/").await;
    assert!(st == StatusCode::INTERNAL_SERVER_ERROR && body.contains("Something failed"));
}
