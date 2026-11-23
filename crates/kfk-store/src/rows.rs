//! SQLite rows into model structs
use crate::model::*;
use kfk_usage as usage;
use rusqlite::Row;

pub(crate) fn account(r: &Row) -> rusqlite::Result<Account> {
    Ok(Account {
        slot: r.get("slot")?,
        label: r.get::<_, Option<String>>("label")?.unwrap_or_default(),
        minted: r.get::<_, Option<String>>("minted")?.unwrap_or_default(),
        off: r.get::<_, i64>("off")? != 0,
        wins: usage::parse(&r.get::<_, Option<String>>("windows")?.unwrap_or_default()),
        asof: r.get("asof")?,
    })
}

pub(crate) fn seat(r: &Row) -> rusqlite::Result<Seat> {
    Ok(Seat {
        seat: r.get("seat")?,
        role: r.get::<_, Option<String>>("role")?.unwrap_or_default(),
        slot: r.get("slot")?,
        sid: r.get("sid")?,
        pid: r.get("pid")?,
        since: r.get("since")?,
        ctx: r.get("ctx")?,
        model: r.get("model")?,
        effort: r.get("effort")?,
        state: r.get("state")?,
        restarts: serde_json::from_str(&r.get::<_, String>("restarts")?).unwrap_or_default(),
        idle_since: r.get("idle_since")?,
        note: r.get::<_, Option<String>>("note")?.unwrap_or_default(),
    })
}

pub(crate) fn session(r: &Row) -> rusqlite::Result<Session> {
    Ok(Session {
        sid: r.get("sid")?,
        seat: r.get("seat")?,
        role: r.get("role")?,
        slot: r.get("slot")?,
        model: r.get("model")?,
        effort: r.get("effort")?,
        started: r.get::<_, Option<f64>>("started")?.unwrap_or(0.0),
        ended: r.get("ended")?,
        path: r.get("path")?,
        bytes_done: r.get("bytes_done")?,
        last: r.get("last")?,
    })
}

pub(crate) fn spend(r: &Row) -> rusqlite::Result<Spend> {
    Ok(Spend { seat: r.get(0)?, role: r.get(1)?, in_tok: r.get(2)?, cw_tok: r.get(3)?, cr_tok: r.get(4)?, out_tok: r.get(5)?, turns: r.get(6)? })
}

pub(crate) fn task(r: &Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: r.get("id")?,
        parent: r.get("parent")?,
        owner: r.get("owner")?,
        title: r.get("title")?,
        spec: r.get("spec")?,
        dod: r.get("dod")?,
        chk: r.get("chk")?,
        repo: r.get("repo")?,
        base: r.get("base")?,
        class: r.get("class")?,
        model: r.get("model")?,
        effort: r.get("effort")?,
        timeout_s: r.get("timeout_s")?,
        max_attempts: r.get("max_attempts")?,
        deps: r.get("deps")?,
        status: r.get("status")?,
        attempts: r.get("attempts")?,
        runner: r.get("runner")?,
        result: r.get("result")?,
        created: r.get::<_, Option<f64>>("created")?.unwrap_or(0.0),
        updated: r.get::<_, Option<f64>>("updated")?.unwrap_or(0.0),
    })
}

pub(crate) fn run(r: &Row) -> rusqlite::Result<Run> {
    Ok(Run {
        id: r.get("id")?,
        task: r.get("task")?,
        attempt: r.get("attempt")?,
        slot: r.get("slot")?,
        model: r.get("model")?,
        effort: r.get("effort")?,
        pid: r.get("pid")?,
        status: r.get("status")?,
        started: r.get::<_, Option<f64>>("started")?.unwrap_or(0.0),
        ended: r.get("ended")?,
        dir: r.get("dir")?,
        wd: r.get("wd")?,
        check_rc: r.get("check_rc")?,
        in_tok: r.get("in_tok")?,
        cw_tok: r.get("cw_tok")?,
        cr_tok: r.get("cr_tok")?,
        out_tok: r.get("out_tok")?,
        cost: r.get("cost")?,
        turns: r.get("turns")?,
        note: r.get("note")?,
    })
}
