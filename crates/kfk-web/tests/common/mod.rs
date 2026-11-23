#![allow(dead_code)]
use anyhow::Result;
use kfk_store::{Gate, Seat, Store, Task};
use kfk_web::probe::Probe;

/// a probe with no tmux and no PostgreSQL behind it
pub struct Fake {
    pub up: bool,
    pub pane: &'static str,
    pub archive: Option<Vec<String>>,
}

impl Default for Fake {
    fn default() -> Self {
        Fake { up: true, pane: "\n ❯ working on T1 \n\n  cos | Fable | ctx 10k\n\n", archive: None }
    }
}

impl Probe for Fake {
    fn fleet_up(&self, _: &Store) -> bool {
        self.up
    }
    fn mizan(&self, _: &Store) -> bool {
        self.up
    }
    fn node_state(&self, _: &Store, seat: &Seat) -> String {
        if seat.state == "up" && self.up { "up".into() } else { "stopped".into() }
    }
    fn pane(&self, _: &Store, _: &str) -> String {
        self.pane.into()
    }
    fn archived(&self, _: &Store, path: &str) -> Result<Vec<String>> {
        self.archive.clone().ok_or_else(|| anyhow::anyhow!("{path} is neither on disk nor in the log archive"))
    }
}

pub fn task(s: &Store, title: &str, parent: Option<i64>, owner: &str) -> i64 {
    s.add_task(&Task { title: title.into(), parent, spec: "the spec".into(), dod: "the dod".into(), chk: "true".into(), class: "impl".into(), owner: owner.into(), ..Default::default() }).unwrap()
}

/// one finished run of a task, with its files; returns the run id
pub fn run(s: &Store, tid: i64, status: &str) -> i64 {
    let Gate::Started { run, .. } = s.begin_run(tid, 1, "sonnet", "medium", &[]).unwrap() else { panic!() };
    let dir = format!("{}/runs/{run}", s.kh);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(format!("{dir}/prompt.md"), "the prompt").unwrap();
    std::fs::write(format!("{dir}/result.md"), "last words").unwrap();
    std::fs::write(format!("{dir}/check.log"), "check said no").unwrap();
    std::fs::write(format!("{dir}/stream.jsonl"), "{\"type\":\"system\",\"subtype\":\"init\"}\n{\"type\":\"assistant\",\"timestamp\":\"2026-11-22T10:00:00Z\",\"message\":{\"content\":[{\"type\":\"text\",\"text\":\"I did <it>\"},{\"type\":\"tool_use\",\"name\":\"Bash\",\"input\":{\"command\":\"make test\"}}]}}\n{\"type\":\"rate_limit_event\"}\n{\"type\":\"result\",\"result\":\"done\"}\n").unwrap();
    s.run_place(run, &dir, "/wt").unwrap();
    if status != "running" {
        s.run_end(run, status, Some((status != "pass") as i64), "the verdict").unwrap();
    }
    run
}

/// a store with a fleet in it: cos, one busy orchestrator, a stopped one, an account pool, streams in every lane
pub struct World {
    pub s: Store,
    pub flight: i64,
    pub kid: i64,
    pub failed_kid: i64,
    pub run: i64,
    pub ask: i64,
}

pub fn world() -> World {
    let s = Store::temp();
    s.put_account(1, "one@x", "2026-11-22").unwrap();
    s.put_account(2, "two@x", "2026-11-22").unwrap();
    s.set_off(2, true).unwrap();
    for (name, role, model) in [("cos", "cos", "fable"), ("o1", "orch", "opus"), ("o2", "orch", "sonnet")] {
        s.put_seat(name, role, model, "high").unwrap();
        s.seat_slot(name, 1).unwrap();
        s.seat_started(name, &format!("sid-{name}"), 1).unwrap();
        s.session_open(&format!("sid-{name}"), name, role, 1, model, "high").unwrap();
    }
    s.seat_ctx("cos", 50_000).unwrap();
    s.seat_note("cos", "waiting for the owner: accept the folder trust dialog (kfk attach)").unwrap();
    s.seat_state("o2", "stopped").unwrap();
    s.session_end("sid-o2").unwrap();
    let hour = (kfk_util::now() / 3600.0) as i64;
    s.session_progress("sid-cos", &format!("{}/cos.jsonl", s.kh), 10, "", &[(hour, [1000, 0, 0, 10], 4), (hour - 2, [500, 0, 0, 0], 1)]).unwrap();
    std::fs::write(format!("{}/cos.jsonl", s.kh), "{\"type\":\"user\",\"timestamp\":\"2026-11-22T09:00:00Z\",\"message\":{\"content\":\"the goal\"}}\n").unwrap();
    let flight = task(&s, "stream in flight", None, "o1");
    let kid = task(&s, "a running child", Some(flight), "o1");
    let failed_kid = task(&s, "a failed child", Some(flight), "o1");
    s.set_field(failed_kid, "deps", &kid.to_string()).unwrap();
    s.claim(kid).unwrap();
    let _live = run(&s, kid, "running");
    s.claim(failed_kid).unwrap();
    let r = run(&s, failed_kid, "fail");
    s.finish(failed_kid, "failed", "check still failing after 1 attempts").unwrap();
    s.event(Some(failed_kid), "note", "a note from o1");
    let _planned = task(&s, "stream planned", None, "o1");
    let queued = task(&s, "stream queued", None, "o1");
    task(&s, "waiting child", Some(queued), "o1");
    let stuck = task(&s, "stream stuck", None, "");
    s.set_status(stuck, "failed", Some("it broke")).unwrap();
    let ask = task(&s, "stream that asks", None, "o1");
    let ask_kid = task(&s, "asking child", Some(ask), "o1");
    s.set_status(ask_kid, "blocked", Some("needs owner: which region?")).unwrap();
    let done = task(&s, "stream done", None, "o1");
    s.set_status(done, "done", Some("branch x")).unwrap();
    let dropped = task(&s, "stream dropped", None, "o1");
    s.set_status(dropped, "cancelled", None).unwrap();
    let w: kfk_usage::Wins = [("seven_day".to_string(), kfk_usage::Win { util: Some(97.0), reset: Some(kfk_util::now() + 3600.0), status: None }), ("five_hour".to_string(), kfk_usage::Win { util: None, reset: None, status: Some("rejected".into()) }), ("old".to_string(), kfk_usage::Win { util: Some(5.0), reset: Some(1.0), status: None })].into_iter().collect();
    s.record_windows(1, &w, "fable").unwrap();
    World { s, flight, kid, failed_kid, run: r, ask }
}
