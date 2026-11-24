//! the command line
use clap::{Args, Parser, Subcommand};

/// a task id: `7` or `T7`
pub fn tid(s: &str) -> Result<i64, String> {
    s.trim_start_matches(['T', 't']).parse().map_err(|_| format!("`{s}` is not a task id (7 or T7)"))
}

#[derive(Parser)]
#[command(name = "kfk", version, about = "kun fayakoon: a chief of staff, orchestrators on demand, short-lived agents, one local state store")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// start cos (on the best account) and mizan
    Up {
        #[arg(long)] dir: Option<String>,
        /// give cos a new conversation instead of resuming its last one
        #[arg(long)] fresh: bool,
    },
    /// stop mizan and every node; tasks, accounts and history are untouched
    Down { #[arg(long)] force: bool },
    /// open the tmux session on cos
    Attach,
    /// nodes, queues, running agents, the last 24 hours
    Status { #[arg(long)] json: bool },
    /// delivery and token numbers
    Report { #[arg(long, default_value_t = 1.0)] days: f64 },
    /// mint and store an account token: one slot, a list (3,12) or a range (10-20)
    Auth { slots: String, #[arg(long)] paste: bool, #[arg(long)] from_log: Option<String>, #[arg(long)] label: Option<String> },
    /// the account pool
    #[command(alias = "ls")]
    Accounts,
    /// one small call per slot: does its token still work
    Check { slots: Option<String> },
    /// never choose this account for a new agent or seat
    Off { slot: i64 },
    /// back in the pool
    On { slot: i64 },
    /// remove an account from the pool
    Rm { slot: i64, #[arg(long)] yes: bool },
    /// move a node to another account (default: the best one); its session is kept
    Swap { seat: String, slot: Option<i64>, #[arg(long)] no_restart: bool },
    /// seats
    Seat { #[command(subcommand)] cmd: Option<SeatCmd> },
    /// orchestrators, started on demand
    Orch { #[command(subcommand)] cmd: OrchCmd },
    /// run a command on a seat's or slot's account
    As { who: String, #[arg(trailing_var_arg = true, required = true)] argv: Vec<String> },
    /// streams and micro-tasks
    Task { #[command(subcommand)] cmd: Option<TaskCmd> },
    /// run one task: an agent per attempt, the check decides
    Run { #[arg(value_parser = tid)] id: i64, #[arg(long)] detach: bool },
    /// run every open child of a stream, in dependency order, in parallel
    Exec { #[arg(value_parser = tid)] id: i64, #[arg(long)] jobs: Option<usize>, #[arg(long)] detach: bool },
    /// block until a task settles
    Wait { #[arg(value_parser = tid)] id: i64 },
    /// the seat layer (no LLM): start | stop | status | once
    Mizan { #[arg(default_value = "status")] action: String },
    /// the PostgreSQL log archive: init | sync | status | stop | cat <path> (an archived file, back on stdout)
    Logs { #[arg(default_value = "status")] action: String, path: Option<String> },
    /// the web interface (read-only): start | stop | status | run
    Web { #[arg(default_value = "status")] action: String },
    /// show or set configuration
    Config { key: Option<String>, value: Option<String> },
    #[command(name = "_node", hide = true)]
    Node { seat: String, #[arg(long)] resume: Option<String> },
    #[command(name = "_statusline", hide = true)]
    Statusline,
    #[command(name = "_mizan", hide = true)]
    MizanLoop,
    #[command(name = "_web", hide = true)]
    WebLoop,
    #[command(name = "_runner", hide = true)]
    Runner { #[arg(value_parser = tid)] id: i64 },
    #[command(name = "_exec", hide = true)]
    ExecLoop { #[arg(value_parser = tid)] id: i64, jobs: usize },
}

#[derive(Subcommand)]
pub enum SeatCmd {
    /// seats and their accounts
    Ls,
    /// put a seat on a slot; takes effect at its next start
    Set { seat: String, slot: i64 },
}

#[derive(Subcommand)]
pub enum OrchCmd {
    /// start an orchestrator; prints its name
    Start { #[arg(long)] model: Option<String>, #[arg(long)] effort: Option<String> },
    /// stop an orchestrator; its tasks stay
    Stop { name: String },
    /// live orchestrators and their queues
    Ls,
}

#[derive(Args)]
pub struct AddArgs {
    pub title: String,
    /// the stream this micro-task belongs to
    #[arg(long, value_parser = tid)] pub parent: Option<i64>,
    /// the orchestrator that owns this stream (o1..)
    #[arg(long)] pub owner: Option<String>,
    /// what and why; text, `-` for stdin, or @file
    #[arg(long)] pub spec: Option<String>,
    /// the observable end state
    #[arg(long)] pub dod: Option<String>,
    /// a shell command; exit status 0 means done
    #[arg(long)] pub check: Option<String>,
    /// repository path (default: the project dir)
    #[arg(long)] pub repo: Option<String>,
    /// branch the work starts from (default: the repo HEAD)
    #[arg(long)] pub base: Option<String>,
    /// mech | impl | hard | deep
    #[arg(long, default_value = "impl")] pub class: String,
    #[arg(long)] pub model: Option<String>,
    #[arg(long)] pub effort: Option<String>,
    /// seconds per attempt
    #[arg(long)] pub timeout: Option<i64>,
    #[arg(long)] pub max_attempts: Option<i64>,
    /// a task that must be done first (repeatable)
    #[arg(long, value_parser = tid)] pub dep: Vec<i64>,
}

#[derive(Args, Default)]
pub struct LsArgs {
    #[arg(long)] pub owner: Option<String>,
    #[arg(long, value_parser = tid)] pub parent: Option<i64>,
    #[arg(long)] pub status: Option<String>,
    #[arg(long)] pub open: bool,
    #[arg(long)] pub streams: bool,
    #[arg(long)] pub json: bool,
    #[arg(long, default_value_t = 60)] pub limit: i64,
}

#[derive(Subcommand)]
pub enum TaskCmd {
    /// add a stream (--owner) or a micro-task (--parent); prints T<id>
    Add(Box<AddArgs>),
    /// list tasks
    Ls(LsArgs),
    /// the task, its children, attempts and the tail of the last failing check
    Show { #[arg(value_parser = tid)] id: i64, #[arg(long)] json: bool },
    /// key=value ... (title spec dod check repo base class model effort timeout max_attempts owner deps)
    Set { #[arg(value_parser = tid)] id: i64, #[arg(required = true)] kv: Vec<String> },
    /// append a decision or fact to the task
    Note { #[arg(value_parser = tid)] id: i64, text: String },
    /// mark done; kfk runs the task's check first and refuses a failure
    Done { #[arg(value_parser = tid)] id: i64, #[arg(long)] result: Option<String>, #[arg(long)] force: bool },
    /// mark failed, with the reason
    Fail { #[arg(value_parser = tid)] id: i64, #[arg(long)] result: Option<String> },
    /// waiting on something outside kfk (for example the owner)
    Block { #[arg(value_parser = tid)] id: i64, #[arg(long)] result: Option<String> },
    /// drop the task
    Cancel { #[arg(value_parser = tid)] id: i64, #[arg(long)] result: Option<String> },
    /// back to todo
    Reopen { #[arg(value_parser = tid)] id: i64 },
}
