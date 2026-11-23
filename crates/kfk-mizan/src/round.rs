//! one round of mizan
use crate::prelude::*;

/// one round of mizan; returns what it did
pub fn round(s: &Store, vault: &dyn Vault) -> Result<Vec<String>> {
    let mut acts = vec![];
    let (n, kill) = s.reconcile(&alive)?;
    for p in kill {
        unsafe { libc::killpg(p as i32, libc::SIGTERM) };
    }
    if n > 0 {
        acts.push(format!("requeued {n} task(s) whose runner died"));
    }
    if session_up(s) {
        for seat in s.seats()? {
            let st = node_state(s, &seat);
            let act = match st {
                "stopped" => None,
                "up" => tend(s, vault, &seat)?,
                _ => revive(s, &seat, st)?,
            };
            acts.extend(act);
        }
    }
    if let Err(e) = fold_sessions(s) {
        acts.push(format!("node spend not read: {e}"));
    }
    Ok(acts)
}

fn recent(seat: &Seat, t: f64) -> Vec<f64> {
    seat.restarts.iter().copied().filter(|x| *x > t - 3600.0).collect()
}

/// a node whose claude or window is gone is started again, at most 3 times an hour
fn revive(s: &Store, seat: &Seat, st: &str) -> Result<Option<String>> {
    let (t, mut hist) = (now(), recent(seat, now()));
    if t - seat.since.unwrap_or(0.0) < 20.0 || hist.len() >= 3 {
        return Ok(None);
    }
    hist.push(t);
    s.seat_restarts(&seat.seat, &hist)?;
    start_node(s, &seat.seat, if seat.sid.is_some() && hist.len() < 3 { Some("it had exited") } else { None })?;
    Ok(Some(format!("restarted {} ({st})", seat.seat)))
}

/// a live node: stop it when idle, note what its pane waits for, move it off an account at its limit
fn tend(s: &Store, vault: &dyn Vault, seat: &Seat) -> Result<Option<String>> {
    if let Some(act) = idle(s, seat)? {
        return Ok(Some(act));
    }
    let wait = pane_wait(&capture(s, &seat.seat));
    let note = wait.map(Wait::note).unwrap_or("");
    if note != seat.note {
        s.seat_note(&seat.seat, note)?;
    }
    let (t, lim) = (now(), s.cfgi("node_max_util") as f64);
    let u = s.account(seat.slot.unwrap_or(LOGIN))?.and_then(|a| usage::util(&a.wins, &seat.model, t).0);
    let why = match (u, wait) {
        (Some(u), _) if u >= lim => format!("its account reached {}% of a usage window", u as i64),
        (_, Some(Wait::Limit)) => "its account hit a usage limit".to_string(),
        _ => return Ok(None),
    };
    let mut hist = recent(seat, t);
    if hist.len() >= 3 {
        return Ok(None);
    }
    let Some(new) = s.pick_slot(&seat.model, &seat.slot.into_iter().collect::<Vec<_>>(), (lim - 15.0).max(1.0), &HashMap::new())?.filter(|x| *x != LOGIN) else { return Ok(None) };
    hist.push(t);
    s.seat_restarts(&seat.seat, &hist)?;
    let msg = move_seat(s, vault, &seat.seat, Some(new), true, &why)?;
    s.event(None, "rotate", &msg);
    Ok(Some(format!("rotated {msg}")))
}

/// an orchestrator with no open task for `orch_idle_s` is stopped
fn idle(s: &Store, seat: &Seat) -> Result<Option<String>> {
    if seat.role != "orch" {
        return Ok(None);
    }
    if s.busy(&seat.seat)? {
        if seat.idle_since.is_some() {
            s.seat_idle(&seat.seat, None)?;
        }
        return Ok(None);
    }
    let (t, max) = (now(), s.cfgi("orch_idle_s") as f64);
    let since = seat.idle_since.unwrap_or(t);
    if seat.idle_since.is_none() {
        s.seat_idle(&seat.seat, Some(t))?;
    }
    if t - since < max || t - seat.since.unwrap_or(t) < max {
        return Ok(None);
    }
    stop_node(s, &seat.seat, "idle")?;
    Ok(Some(format!("stopped {} (no open task for {}s)", seat.seat, (t - since) as i64)))
}
