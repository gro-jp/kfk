//! the WebSocket of a terminal: browser on one side, a tmux client on a pty on the other
use crate::routes::App;
use crate::shell::{shell_open, Open};
use axum::extract::ws::{rejection::WebSocketUpgradeRejection, Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use kfk_store::Store;
use kfk_term::{resize, size, Pty};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(1);

fn refuse(status: u16, why: String) -> Response {
    (StatusCode::from_u16(status).unwrap_or(StatusCode::FORBIDDEN), why).into_response()
}

pub async fn ws_node(State(a): State<App>, Path(seat): Path<String>, Query(q): Query<HashMap<String, String>>, h: HeaderMap, ws: Result<WebSocketUpgrade, WebSocketUpgradeRejection>) -> Response {
    let head = |n: header::HeaderName| h.get(n).and_then(|v| v.to_str().ok()).map(String::from);
    let (host, origin) = (head(header::HOST), head(header::ORIGIN));
    let id = format!("{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed));
    let decided = tokio::task::spawn_blocking(move || {
        let s = Store::open(&a.kh).map_err(|e| (500, e.to_string()))?;
        let socket = std::env::var("KFK_TMUX_SOCKET").ok();
        shell_open(&s, a.probe.as_ref(), &Open { local: a.local, seat: &seat, host: host.as_deref(), origin: origin.as_deref(), socket: socket.as_deref(), id: &id })
    })
    .await
    .unwrap_or_else(|_| Err((500, "the terminal could not be opened".into())));
    let args = match decided {
        Ok(a) => a,
        Err((status, why)) => return refuse(status, why),
    };
    let num = |k: &str| q.get(k).and_then(|x| x.parse::<u64>().ok());
    let (cols, rows) = size(num("cols"), num("rows"));
    match ws {
        Ok(up) => up.on_upgrade(move |sock| bridge(sock, args, cols, rows)),
        Err(_) => refuse(426, "This address is a terminal; it answers only a WebSocket.".into()),
    }
}

async fn bridge(mut sock: WebSocket, args: Vec<String>, cols: u16, rows: u16) {
    let opened = Pty::spawn("tmux", &args, cols, rows).and_then(|p| Ok((p.reader()?, p.writer()?, p)));
    let (mut from, mut to, pty) = match opened {
        Ok(x) => x,
        Err(e) => {
            let _ = sock.send(Message::Text(format!("kfk: {e}\r\n").into())).await;
            return;
        }
    };
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(64);
    std::thread::spawn(move || {
        let mut b = [0u8; 16384];
        while let Ok(n) = from.read(&mut b) {
            if n == 0 || tx.blocking_send(b[..n].to_vec()).is_err() {
                break;
            }
        }
    });
    loop {
        tokio::select! {
            out = rx.recv() => match out {
                Some(b) => if sock.send(Message::Binary(b.into())).await.is_err() { break },
                None => break,
            },
            msg = sock.recv() => match msg {
                Some(Ok(Message::Binary(b))) => if to.write_all(&b).is_err() { break },
                Some(Ok(Message::Text(t))) => if let Some((c, r)) = resize(&t) { pty.resize(c, r) },
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                Some(Ok(_)) => {}
            },
        }
    }
    // dropping the pty ends the tmux client; its session goes with it, the node's window stays
    drop(pty);
}
