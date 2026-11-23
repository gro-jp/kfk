//! kfk (kun fayakoon): one chief of staff, orchestrators on demand, short-lived agents, one local state store.
mod cli;
mod cmd;
mod ctx;

use clap::Parser;

fn main() {
    // pipe-able: a closed pipe (| head) ends kfk quietly instead of panicking
    unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL) };
    let a = cli::Cli::parse();
    match ctx::Ctx::open().and_then(|c| cmd::dispatch(&c, a.cmd)) {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("kfk: {e}");
            std::process::exit(1);
        }
    }
}
