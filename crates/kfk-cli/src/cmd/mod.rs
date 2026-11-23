//! one file per group of commands
mod accounts;
mod auth;
mod config;
mod fleet;
mod hidden;
mod logs;
mod report;
mod run;
mod seats;
mod status;
mod task;
mod task_show;
mod web;

use crate::cli::Cmd;
use crate::ctx::Ctx;
use anyhow::Result;

pub fn dispatch(c: &Ctx, cmd: Cmd) -> Result<i32> {
    match cmd {
        Cmd::Up { dir, fresh } => fleet::up(c, dir, fresh),
        Cmd::Down { force } => fleet::down(c, force),
        Cmd::Attach => fleet::attach(c),
        Cmd::Orch { cmd } => fleet::orch(c, cmd),
        Cmd::Mizan { action } => fleet::mizan(c, &action),
        Cmd::Status { json } => status::status(c, json),
        Cmd::Report { days } => report::report(c, days),
        Cmd::Auth { slots, paste, from_log, label } => auth::auth(c, &slots, paste, from_log, label),
        Cmd::Accounts => accounts::list(c),
        Cmd::Check { slots } => accounts::check(c, slots),
        Cmd::Off { slot } => accounts::onoff(c, slot, true),
        Cmd::On { slot } => accounts::onoff(c, slot, false),
        Cmd::Rm { slot, yes } => accounts::rm(c, slot, yes),
        Cmd::Swap { seat, slot, no_restart } => seats::swap(c, &seat, slot, !no_restart),
        Cmd::Seat { cmd } => seats::seat(c, cmd),
        Cmd::As { who, argv } => seats::run_as(c, &who, &argv),
        Cmd::Task { cmd } => task::task(c, cmd),
        Cmd::Run { id, detach } => run::run(c, id, detach),
        Cmd::Exec { id, jobs, detach } => run::exec(c, id, jobs, detach),
        Cmd::Wait { id } => run::wait(c, id),
        Cmd::Logs { action, path } => logs::logs(c, &action, path.as_deref()),
        Cmd::Web { action } => web::web(c, &action),
        Cmd::WebLoop => web::web(c, "run"),
        Cmd::Config { key, value } => config::config(c, key, value),
        Cmd::Node { seat, resume } => hidden::node(seat, resume),
        Cmd::Statusline => hidden::statusline(c),
        Cmd::MizanLoop => hidden::mizan_loop(c),
        Cmd::Runner { id } => hidden::runner(c, id),
        Cmd::ExecLoop { id, jobs } => hidden::exec_loop(c, id, jobs),
    }
}
