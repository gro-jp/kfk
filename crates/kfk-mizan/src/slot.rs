//! which account a seat is on
use crate::prelude::*;

pub fn usable(s: &Store, vault: &dyn Vault, slot: Option<i64>) -> bool {
    match slot {
        None => false,
        Some(LOGIN) => s.pool().map(|p| p.is_empty()).unwrap_or(true),
        Some(n) => vault.get(n).is_some() && s.account(n).ok().flatten().map(|a| !a.off).unwrap_or(false),
    }
}

/// give the seat an account if it has none it can use; returns its slot
pub fn ensure_slot(s: &Store, vault: &dyn Vault, seat: &Seat) -> Result<i64> {
    if usable(s, vault, seat.slot) {
        return Ok(seat.slot.unwrap());
    }
    let none = HashMap::new();
    let slot = match s.pick_slot(&seat.model, &[], s.cfgi("node_max_util") as f64, &none)? {
        Some(x) => x,
        None => s.pick_slot(&seat.model, &[], 101.0, &none)?.ok_or_else(|| anyhow!("no account to put {} on", seat.seat))?,
    };
    s.seat_slot(&seat.seat, slot)?;
    Ok(slot)
}

/// put a seat on another account (`slot` None: the best one); a live node is restarted with its session kept
pub fn move_seat(s: &Store, vault: &dyn Vault, name: &str, slot: Option<i64>, restart: bool, why: &str) -> Result<String> {
    let seat = s.seat(name)?.ok_or_else(|| anyhow!("no seat {name} (kfk seat ls)"))?;
    let slot = match slot {
        Some(x) if x == LOGIN || vault.get(x).is_some() => x,
        Some(x) => bail!("slot {x} has no token (kfk auth {x})"),
        None => s.pick_slot(&seat.model, &seat.slot.into_iter().collect::<Vec<_>>(), s.cfgi("node_max_util") as f64, &HashMap::new())?.ok_or_else(|| anyhow!("no other account has headroom for {name} (kfk accounts)"))?,
    };
    s.seat_slot(name, slot)?;
    s.event(None, "swap", &format!("{name}: {:?} -> {slot} ({why})", seat.slot));
    let live = node_state(s, &seat) == "up";
    if live && restart {
        start_node(s, name, Some(why))?;
    }
    let show = |x: Option<i64>| match x { None | Some(LOGIN) => "login".to_string(), Some(n) => n.to_string() };
    Ok(format!("{name}: slot {} -> {}{}", show(seat.slot), show(Some(slot)), if live && restart { " | restarted, session kept" } else if live { " | takes effect at its next start" } else { "" }))
}
