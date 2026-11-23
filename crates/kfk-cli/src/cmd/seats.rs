//! seats: which node is on which account
use crate::cli::SeatCmd;
use crate::ctx::{slot_name, Ctx};
use anyhow::{bail, Result};
use kfk_engine::clean_env;
use kfk_usage::{fmt, LOGIN};
use kfk_util::{ago, ktok, now, table};
use std::os::unix::process::CommandExt;

pub fn ls(c: &Ctx) -> Result<i32> {
    let mut rows = vec![];
    for s in c.s.seats()? {
        let a = c.s.account(s.slot.unwrap_or(LOGIN))?;
        rows.push(vec![s.seat.clone(), kfk_mizan::node_state(&c.s, &s).to_string(), slot_name(s.slot), a.as_ref().map(|a| a.label.clone()).unwrap_or_else(|| "?".into()),
                       a.map(|a| fmt(&a.wins, now())).unwrap_or_else(|| "-".into()), format!("{}/{}", s.model, s.effort), s.ctx.map(ktok).unwrap_or_else(|| "-".into()), ago(s.since, now())]);
    }
    println!("{}", table(&["seat", "node", "slot", "account", "usage", "model", "ctx", "since"], &rows));
    Ok(0)
}

pub fn seat(c: &Ctx, cmd: Option<SeatCmd>) -> Result<i32> {
    if let Some(SeatCmd::Set { seat, slot }) = cmd {
        println!("{}", kfk_mizan::move_seat(&c.s, c.v.as_ref(), &seat, Some(slot), false, "seat set")?);
    }
    ls(c)
}

pub fn swap(c: &Ctx, seat: &str, slot: Option<i64>, restart: bool) -> Result<i32> {
    println!("{}", kfk_mizan::move_seat(&c.s, c.v.as_ref(), seat, slot, restart, "kfk swap")?);
    Ok(0)
}

/// run a command on a seat's or slot's account
pub fn run_as(c: &Ctx, who: &str, argv: &[String]) -> Result<i32> {
    let slot = match who.parse::<i64>() {
        Ok(n) => n,
        Err(_) => match c.s.seat(who)? {
            Some(s) => s.slot.unwrap_or(LOGIN),
            None => bail!("{who:?} is neither a slot number nor a seat (kfk seat ls)"),
        },
    };
    let tok = c.v.get(slot);
    if slot != LOGIN && tok.is_none() {
        bail!("slot {slot} has no token (kfk auth {slot})");
    }
    let e = std::process::Command::new(&argv[0]).args(&argv[1..]).env_clear().envs(clean_env(tok.as_deref(), &[("KFK_SLOT", slot.to_string())])).exec();
    bail!("cannot run {}: {e}", argv[0])
}
