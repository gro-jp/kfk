//! the account pool
use crate::ctx::{slot_name, Ctx};
use anyhow::{bail, Result};
use kfk_usage::{fmt, util, LOGIN};
use kfk_util::{ago, now, parse_slots, table, until};
use std::collections::HashMap;

pub fn list(c: &Ctx) -> Result<i32> {
    let (mut seats, mut live): (HashMap<i64, Vec<String>>, HashMap<i64, i64>) = (HashMap::new(), HashMap::new());
    for s in c.s.seats()?.into_iter().filter(|s| s.state == "up") {
        seats.entry(s.slot.unwrap_or(LOGIN)).or_default().push(s.seat);
    }
    for r in c.s.running()? {
        *live.entry(r.slot).or_default() += 1;
    }
    let pool = c.s.pool()?;
    let mut all = pool.clone();
    if pool.is_empty() || seats.contains_key(&LOGIN) || live.contains_key(&LOGIN) {
        all.extend(c.s.account(LOGIN)?);
    }
    let rows: Vec<Vec<String>> = all.iter().map(|a| {
        let state = if a.off { "off" } else if a.slot == LOGIN || c.v.get(a.slot).is_some() { "ok" } else { "NO TOKEN" };
        let dash = |x: &str| if x.is_empty() { "-".to_string() } else { x.to_string() };
        vec![slot_name(Some(a.slot)), dash(&a.label), dash(&a.minted), state.into(), fmt(&a.wins, now()), until(util(&a.wins, "", now()).1), ago(a.asof, now()),
             dash(&seats.get(&a.slot).map(|v| v.join(",")).unwrap_or_default()), live.get(&a.slot).map(|n| n.to_string()).unwrap_or_else(|| "-".into())]
    }).collect();
    println!("{}", table(&["slot", "account", "minted", "state", "usage", "resets", "seen", "seats", "agents"], &rows));
    if pool.is_empty() {
        println!("\nno accounts in the pool: everything runs on the saved login. Add one: kfk auth 0");
    }
    Ok(0)
}

pub fn check(c: &Ctx, slots: Option<String>) -> Result<i32> {
    let mut list = match slots.as_deref() {
        None | Some("all") => c.s.pool()?.iter().map(|a| a.slot).collect(),
        Some(s) => parse_slots(s)?,
    };
    if list.is_empty() {
        list.push(LOGIN);
    }
    let mut bad = 0;
    for slot in list {
        let (rc, o) = kfk_engine::ping(c.v.get(slot).as_deref());
        c.s.record_windows(slot, &o.wins, "haiku")?;
        println!("slot {}: {}", slot_name(Some(slot)), match rc { 0 => "ok".to_string(), 1 => format!("REJECTED (mint again: kfk auth {slot})"), _ => "call failed (not an auth error)".to_string() });
        bad += (rc != 0) as i32;
    }
    Ok(if bad > 0 { 1 } else { 0 })
}

pub fn onoff(c: &Ctx, slot: i64, off: bool) -> Result<i32> {
    if !c.s.set_off(slot, off)? {
        bail!("slot {slot} holds no account");
    }
    c.s.event(None, if off { "off" } else { "on" }, &format!("slot {slot}"));
    println!("slot {slot} is {}", if off { "OFF: never chosen for a new agent or seat; a seat already on it keeps it" } else { "ON: back in the pool" });
    Ok(0)
}

pub fn rm(c: &Ctx, slot: i64, yes: bool) -> Result<i32> {
    let Some(a) = c.s.account(slot)?.filter(|a| a.slot >= 0) else { bail!("slot {slot} holds no account") };
    let seats: Vec<String> = c.s.seats()?.into_iter().filter(|s| s.slot == Some(slot)).map(|s| s.seat).collect();
    println!("remove slot {slot} = {}; seats on it: {}. The token itself stays valid until it expires; only a new sign-in brings it back.", a.label, if seats.is_empty() { "none".into() } else { seats.join(", ") });
    if !yes {
        let mut ans = String::new();
        println!("type yes to remove:");
        std::io::stdin().read_line(&mut ans)?;
        if ans.trim() != "yes" {
            bail!("nothing removed");
        }
    }
    c.v.del(slot);
    c.s.del_account(slot)?;
    c.s.event(None, "rm", &format!("slot {slot} {}", a.label));
    for s in seats {
        println!("{}", kfk_mizan::move_seat(&c.s, c.v.as_ref(), &s, None, true, "its account was removed")?);
    }
    println!("slot {slot} removed");
    Ok(0)
}
