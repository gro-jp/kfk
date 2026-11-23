# Architecture

kfk is one binary built from a workspace of small crates. Each crate does one thing; no source file is longer than
about 160 lines.

## Processes

```
kfk up
 ├─ tmux session "kfk"
 │   ├─ window cos   kfk _node cos ──▶ claude (interactive)
 │   ├─ window o1    kfk _node o1  ──▶ claude (interactive)
 │   └─ ...
 ├─ kfk _mizan       the round loop, every 30 s
 └─ kfk _web         the web view

kfk exec <stream>    (run by an orchestrator, or by you)
 └─ kfk _exec        detached; starts runners as dependencies allow
     ├─ kfk _runner T8  ──▶ claude -p ... ──▶ bash -c '<check>'      (repeat per attempt)
     └─ kfk _runner T9  ──▶ ...

PostgreSQL           started by `kfk logs init|sync`; a private cluster in the data home
```

Every long-running piece is the same `kfk` binary started with a hidden subcommand (`_node`, `_mizan`, `_web`,
`_exec`, `_runner`, `_statusline`). Runners and the exec loop are started in their own session, so they outlive the
terminal, the orchestrator and `kfk down`.

All of them share state through one SQLite file in WAL mode. There is no server between them. The concurrency gate
and the account choice run inside a single `begin immediate` transaction, which is what makes parallel runners
safe.

## Crates

| Crate | One thing | Depends on (kfk crates) |
| --- | --- | --- |
| `kfk-util` | Small helpers: time, formatting, tables, slot lists, the data-home rule | – |
| `kfk-usage` | Usage windows per account; choosing the account with most headroom | – |
| `kfk-route` | Model and effort for an attempt, from a ladder | – |
| `kfk-spend` | Folding a transcript into hourly token spend | – |
| `kfk-roles` | The role prompts in `roles/`, compiled in | – |
| `kfk-stream` | Parsing what a `claude -p --output-format stream-json` run left behind | usage, util |
| `kfk-token` | Reading a token back from a sign-in transcript; the vault (Keychain) | – |
| `kfk-store` | The SQLite store: config, accounts, seats, tasks, runs, sessions, events | usage, util, stream |
| `kfk-report` | Delivery and token numbers over a period | store, util |
| `kfk-engine` | One attempt: worktree, prompt, launch, timeout, check; the task runner; the stream executor | store, stream, route, roles, token, usage, util |
| `kfk-logs` | The PostgreSQL archive: find files, sync incrementally, read back | store, util |
| `kfk-mizan` | Nodes in tmux, account moves, the round, the daemon, the status line | engine, logs, spend, roles, store, token, usage, util |
| `kfk-web` | View models, Maud pages, Axum routes, the server | engine, logs, mizan, report, store, usage, util |
| `kfk-cli` | The `kfk` binary: argument parsing and one small file per command group | engine, logs, mizan, web, report, store, token, usage, util |

The lower crates are pure: they take values and return values, which is why they can be held to 100% line
coverage. Code that touches the outside world (tmux, the Keychain, PostgreSQL, sockets, process spawning) is kept
in a few named files; see [Development](development.md).

`kfk-web` reaches tmux, the daemon and PostgreSQL only through a small `Probe` trait, so every page can be rendered
in a test without any of them.

## The data home

```
<data home>/                    default ~/.kfk
├── kfk.db  kfk.db-wal  kfk.db-shm    the store
├── runs/<run id>/              one directory per agent attempt
│   ├── prompt.md                 exactly what the agent was given
│   ├── stream.jsonl              everything it printed (stream-json)
│   ├── result.md                 its last words
│   └── check.log                 the check's output
├── wt/T<id>/                   the git worktree of a task (branch kfk/T<id>)
├── work/T<id>/                 the working directory of a task that has no repository
├── nodes/<seat>.settings.json  the Claude Code settings a node is started with
├── log/                        mizan.log, web.log, pg.log, exec-T<id>.log, T<id>.log
├── done-T<id>.log              the check output from `kfk task done`
├── pg/                         the PostgreSQL cluster of the log archive
├── .s.PGSQL.<port>             its socket
└── mizan.pid  web.pid
```

Nothing in `runs/` is ever deleted by kfk. Worktrees stay until you remove them:

```sh
git -C <repo> worktree remove ~/.kfk/wt/T12 && git -C <repo> branch -D kfk/T12
```

Outside the data home, kfk writes two things: the pointer `~/.config/kfk/home`, and Keychain entries (service
`kfk`). Claude Code keeps its own transcripts in `~/.claude/projects/`.

## The store

`<data home>/kfk.db`, created and migrated on open. Open it read-only with `sqlite3` at any time.

| Table | Rows | Notes |
| --- | --- | --- |
| `config` | `k`, `v` | Only keys you changed; defaults live in the binary. |
| `accounts` | `slot`, `label`, `minted`, `off`, `windows` (JSON), `asof` | Slot `-1` is the saved login. No tokens. |
| `seats` | `seat`, `role`, `slot`, `sid`, `pid`, `since`, `ctx`, `model`, `effort`, `state`, `restarts`, `idle_since`, `note` | One per node name ever used. |
| `tasks` | `id`, `parent`, `owner`, `title`, `spec`, `dod`, `chk`, `repo`, `base`, `class`, `model`, `effort`, `timeout_s`, `max_attempts`, `deps`, `status`, `attempts`, `runner`, `result`, `created`, `updated` | Streams have `parent` null. |
| `runs` | `id`, `task`, `attempt`, `slot`, `model`, `effort`, `pid`, `status`, `started`, `ended`, `dir`, `wd`, `check_rc`, token counts, `cost`, `turns`, `note` | One per attempt. Status: `running`, `pass`, `fail`, `limit`, `error`, `lost`. |
| `sessions` | `sid`, `seat`, `role`, `slot`, `model`, `effort`, `started`, `ended`, `path`, `bytes_done`, `last` | Every Claude session a node has had. |
| `spend` | `sid`, `hour`, token counts, `turns` | Node tokens by hour. |
| `events` | `id`, `ts`, `task`, `actor`, `kind`, `data` | The append-only log: adds, notes, results, starts, stops, swaps, rotations. |

Times are Unix seconds. Rows in `tasks`, `runs` and `events` are never deleted.

```sh
sqlite3 -readonly ~/.kfk/kfk.db \
  "select class, model, count(*), sum(status='pass') from runs join tasks on tasks.id=runs.task group by 1,2"
```

## One attempt, step by step

1. The runner takes the task (`todo` → `running`), exactly once even if two try.
2. It asks the store for a slot: if fewer than `max_agents` runs are live and an account has headroom, a `runs`
   row is inserted and the slot returned, in one transaction.
3. A worktree is created or reused; the prompt is written to `runs/<id>/prompt.md`: the agent role, the task's
   spec, done-when and check, and, after a failed attempt, the tail of that attempt's check output.
4. `claude -p --model <m> --effort <e> --output-format stream-json --verbose <agent_flags>` runs in the worktree
   with a clean environment, the prompt on standard input, output to `stream.jsonl`, in its own process group.
5. If it runs past the time limit, the group is stopped.
6. The stream is parsed: tokens, cost, turns, usage windows, and whether it ended on a limit or an auth error. The
   windows are stored on the account.
7. On a limit or auth error: the run is marked `limit`, the account is set aside, the attempt is not counted, and
   step 2 repeats with that account excluded.
8. Otherwise the check runs in the worktree, output to `check.log`.
9. Exit 0: the run is `pass` and the task `done`. Anything else: the run is `fail`, and the loop continues one rung
   up until `max_attempts`.

## Messages between sessions

Nodes are started with a settings file that allows inbound cross-session messages, and with `-n <seat>` so they can
be addressed by name. cos sends `T7` to `o1`; `o1` answers `T7 done`. kfk itself sends nothing: if a message were
lost, the task state in the store would still be correct, and a restarted node is told to read it.
