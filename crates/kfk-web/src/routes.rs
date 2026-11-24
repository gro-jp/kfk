//! the routes: each one opens the store, builds a view model and renders it
use crate::model as m;
use crate::page as p;
use crate::probe::Probe;
use anyhow::{anyhow, Result};
use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Router};
use kfk_store::Store;
use maud::Markup;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct App {
    pub kh: String,
    pub probe: Arc<dyn Probe>,
    /// the server listens on a loopback address, so terminals may be offered
    pub local: bool,
}

type Q = Query<HashMap<String, String>>;

/// run `f` off the async threads (SQLite, files and tmux block); an error becomes a page, not a blank
async fn render<F>(app: App, f: F) -> Response
where
    F: FnOnce(&Store, &dyn Probe) -> Result<Markup> + Send + 'static,
{
    let out = tokio::task::spawn_blocking(move || Store::open(&app.kh).and_then(|s| f(&s, app.probe.as_ref()))).await;
    match out.ok().unwrap_or(Err(anyhow!("the page could not be built"))) {
        Ok(page) => page.into_response(),
        Err(e) => {
            let (msg, code) = (e.to_string(), if e.to_string().starts_with("no ") { StatusCode::NOT_FOUND } else { StatusCode::INTERNAL_SERVER_ERROR });
            (code, p::error_page(code.as_u16(), &msg)).into_response()
        }
    }
}

/// `7` or `T7`
fn id(s: &str) -> Result<i64> {
    s.trim_start_matches(['T', 't']).parse().map_err(|_| anyhow!("no task or run {s:?}"))
}

fn page_of(q: &HashMap<String, String>) -> (Option<usize>, bool) {
    (q.get("page").and_then(|x| x.parse().ok()), q.get("all").map(|x| x == "1").unwrap_or(false))
}

async fn overview(State(a): State<App>) -> Response {
    render(a, |s, pr| Ok(p::layout("Overview", "/", p::overview_page(&m::overview(s, pr)?)))).await
}
async fn overview_part(State(a): State<App>) -> Response {
    render(a, |s, pr| Ok(p::overview_part(&m::overview(s, pr)?))).await
}
async fn task(State(a): State<App>, Path(t): Path<String>) -> Response {
    render(a, move |s, _| {
        let v = m::task_view(s, id(&t)?)?;
        Ok(p::layout(&format!("T{} {}", v.card.task.id, v.card.task.title), "/", p::task_page(&v)))
    })
    .await
}
async fn task_part(State(a): State<App>, Path(t): Path<String>) -> Response {
    render(a, move |s, _| Ok(p::task_part(&m::task_view(s, id(&t)?)?))).await
}
async fn run(State(a): State<App>, Path(r): Path<String>, Query(q): Q) -> Response {
    render(a, move |s, pr| {
        let (page, all) = page_of(&q);
        Ok(p::layout(&format!("Run {r}"), "/", p::run_page(&m::run_view(s, pr, id(&r)?, page, all)?)))
    })
    .await
}
async fn nodes(State(a): State<App>) -> Response {
    render(a, |s, pr| Ok(p::layout("Nodes", "/nodes", p::nodes_page(&m::nodes(s, pr)?)))).await
}
async fn node(State(a): State<App>, Path(n): Path<String>) -> Response {
    let local = a.local;
    render(a, move |s, pr| Ok(p::layout(&n, "/nodes", p::node_page(&m::node_view(s, pr, local, &n)?)))).await
}
async fn shell(State(a): State<App>, Path(n): Path<String>) -> Response {
    let local = a.local;
    render(a, move |s, pr| Ok(p::shell_page(&crate::shell::shell_view(s, pr, local, &n)?))).await
}
async fn node_part(State(a): State<App>, Path(n): Path<String>) -> Response {
    let local = a.local;
    render(a, move |s, pr| Ok(p::node_part(&m::node_view(s, pr, local, &n)?))).await
}
async fn session(State(a): State<App>, Path(sid): Path<String>, Query(q): Q) -> Response {
    render(a, move |s, pr| {
        let (page, all) = page_of(&q);
        Ok(p::layout("Session", "/nodes", p::session_page(&m::session_view(s, pr, &sid, page, all)?)))
    })
    .await
}
async fn accounts(State(a): State<App>) -> Response {
    render(a, |s, _| Ok(p::layout("Accounts", "/accounts", p::accounts_page(&m::accounts(s)?)))).await
}
async fn history(State(a): State<App>, Query(q): Q) -> Response {
    render(a, move |s, _| Ok(p::layout("History", "/history", p::history_page(&m::history(s, q.get("days").and_then(|d| d.parse().ok()).unwrap_or(1.0), q.get("page").and_then(|n| n.parse().ok()))?)))).await
}
async fn css() -> Response {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8"), (header::CACHE_CONTROL, "no-cache")], include_str!("assets/app.css")).into_response()
}
async fn htmx() -> Response {
    ([(header::CONTENT_TYPE, "text/javascript; charset=utf-8"), (header::CACHE_CONTROL, "max-age=86400")], include_str!("assets/htmx.min.js")).into_response()
}
/// a bundled library is kept by the browser for a day; kfk's own script is always fetched fresh
fn asset(kind: &'static str, keep: bool, body: &'static str) -> Response {
    ([(header::CONTENT_TYPE, kind), (header::CACHE_CONTROL, if keep { "max-age=86400" } else { "no-cache" })], body).into_response()
}
const JS: &str = "text/javascript; charset=utf-8";
async fn xterm() -> Response {
    asset(JS, true, include_str!("assets/xterm.js"))
}
async fn xterm_fit() -> Response {
    asset(JS, true, include_str!("assets/xterm-addon-fit.js"))
}
async fn term_js() -> Response {
    asset(JS, false, include_str!("assets/term.js"))
}
async fn xterm_css() -> Response {
    asset("text/css; charset=utf-8", true, include_str!("assets/xterm.css"))
}
async fn missing() -> Response {
    (StatusCode::NOT_FOUND, p::error_page(404, "There is no such page.")).into_response()
}

pub fn router(app: App) -> Router {
    Router::new()
        .route("/", get(overview))
        .route("/part/overview", get(overview_part))
        .route("/task/{id}", get(task))
        .route("/part/task/{id}", get(task_part))
        .route("/run/{id}", get(run))
        .route("/nodes", get(nodes))
        .route("/node/{seat}", get(node))
        .route("/part/node/{seat}", get(node_part))
        .route("/shell/{seat}", get(shell))
        .route("/ws/node/{seat}", get(crate::ws::ws_node))
        .route("/session/{sid}", get(session))
        .route("/accounts", get(accounts))
        .route("/history", get(history))
        .route("/assets/app.css", get(css))
        .route("/assets/htmx.min.js", get(htmx))
        .route("/assets/xterm.js", get(xterm))
        .route("/assets/xterm-addon-fit.js", get(xterm_fit))
        .route("/assets/xterm.css", get(xterm_css))
        .route("/assets/term.js", get(term_js))
        .route("/healthz", get(|| async { "ok" }))
        .fallback(missing)
        .with_state(app)
}
