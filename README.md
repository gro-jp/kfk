# kfk

**kfk** (*kun fayakoon*, Arabic for "be, and it is") runs a small fleet of [Claude Code](https://claude.com/claude-code)
sessions that delivers checked work instead of conversation.

You talk to one session, the chief of staff (**cos**). It turns a goal into streams of work and starts an
**orchestrator** for each stream only when one is needed. The orchestrator cuts the stream into micro-tasks, and kfk
runs one short-lived **agent** per micro-task in its own git worktree. Every task carries a check command. A task is
done when its check exits 0, and not before.

Underneath sits **mizan**, a layer with no LLM in it. It chooses which Claude account every session runs on, moves a
session to another account before a usage limit, restarts what died and stops what is idle.

```
  you ── kfk attach ──▶ cos                 one long-lived session; plans, never implements
                         │  kfk orch start  (0 to 7, on demand)
                         ▼
                    orchestrator            one per stream; cuts it into micro-tasks
                         │  kfk exec <stream>
                         ▼
        agent   agent   agent   agent       one `claude -p` per attempt, own worktree, exits when done
          │       │       │       │
          ▼       ▼       ▼       ▼
        check   check   check   check       exit 0 = done; a failure retries one model rung up

  mizan (no LLM): account choice · rotation at limits · restarts · idle stop · token accounting
  state: SQLite (tasks, runs, accounts, events)   logs: PostgreSQL (every transcript, kept for ever)
```

## Why

A large standing fleet of agents is good at coordinating and poor at delivering: most of its tokens go into
re-reading long contexts, status messages and hand-off notes. kfk is built on four rules that push the other way:

1. **A check decides, not a report.** Each task has a spec, a definition of done and a shell command. kfk runs the
   command itself after the agent exits.
2. **Agents are short-lived.** One task, one fresh context, then exit. Nothing accumulates.
3. **State lives in a store, not in chat.** Tasks, attempts, results and decisions are rows in SQLite. A message
   between sessions is one line: `T7`.
4. **Everything that can be done without an LLM is.** Account choice, rotation, restarts, concurrency, retries and
   accounting are plain code.

`kfk report` shows what that costs: tokens per passed task, and the share of all tokens spent on coordination
(cos and orchestrators) rather than on the work.

## Requirements

| Needed | Why |
| --- | --- |
| macOS | Account tokens are kept in the macOS Keychain. Linux is not supported yet; see [the FAQ](docs/faq.md). |
| [Claude Code](https://claude.com/claude-code) CLI, signed in | kfk drives `claude`. Developed against version 2.1.330. |
| Rust (stable) and Cargo | To build. Tested with Rust 1.100. |
| `git` | Each task runs in its own worktree. |
| `tmux` | cos and the orchestrators live in one tmux session. |
| PostgreSQL (optional) | Only for the permanent log archive. |

## Install

```sh
git clone https://github.com/gro-jp/kfk.git kfk && cd kfk
./install.sh
```

The installer builds a release binary, puts it in `~/.local/bin/kfk`, and asks two questions:

- **Data home**: where tasks, runs, logs and the databases go. Default `~/.kfk`.
- **PostgreSQL bin directory**: for the log archive. Leave it empty to skip the archive for now.

Both can be given as flags (`./install.sh --home /data/kfk --pg-bin /opt/pg/bin --prefix ~/.local --yes`) and
changed later. See [Getting started](docs/getting-started.md).

## Quick start

```sh
kfk up --dir ~/code/my-repo     # start cos, mizan and the web view
kfk attach                      # talk to cos (a tmux session; detach with Ctrl-b d)
```

Tell cos what you want, in plain words, with the outcome you expect:

> Add rate limiting to the public API: 100 requests per minute per key, a 429 with a Retry-After header when
> exceeded, and tests. Deliver as a pull request.

cos creates a stream, starts an orchestrator, and reports back in a few lines when the stream's check has passed.
Meanwhile:

```sh
kfk status                      # nodes, queues, running agents
kfk task ls                     # streams and micro-tasks
kfk report                      # what passed and what it cost
open http://127.0.0.1:7878      # the same, in a browser, with every transcript
kfk down                        # stop the fleet; tasks and history stay
```

You do not need cos to use the engine. A single task can be run directly:

```sh
kfk task add "fizzbuzz script" --class mech --repo . \
  --spec "Write fizzbuzz.py: prints 1..100 with Fizz, Buzz and FizzBuzz." \
  --dod  "python3 fizzbuzz.py prints 100 lines; line 15 is FizzBuzz" \
  --check 'test "$(python3 fizzbuzz.py | sed -n 15p)" = FizzBuzz'
kfk run 1
```

### More than one Claude account

With no accounts added, everything runs on the login `claude` already has. To spread load over several accounts:

```sh
kfk auth 0        # a browser sign-in; the token goes into the Keychain
kfk auth 1
kfk accounts      # the pool, with each account's usage windows
```

mizan then picks the account with the most headroom for every session and agent. See [Accounts](docs/accounts.md).

## Documentation

| | |
| --- | --- |
| [Getting started](docs/getting-started.md) | Install, first run, first goal, first task by hand |
| [Concepts](docs/concepts.md) | cos, orchestrators, agents, streams, tasks, checks, classes, mizan |
| [Tasks and checks](docs/tasks-and-checks.md) | Writing tasks that pass first time; dependencies; escalation |
| [Accounts](docs/accounts.md) | The pool, sign-in, usage windows, rotation |
| [mizan](docs/mizan.md) | What the zero-LLM layer does each round |
| [Web interface](docs/web.md) | The pages, and the live terminal on each node |
| [Log archive](docs/logs.md) | Keeping every transcript in PostgreSQL |
| [Commands](docs/commands.md) | Every command, flag and exit status |
| [Configuration](docs/configuration.md) | Every config key and environment variable |
| [Architecture](docs/architecture.md) | Crates, the data home, the database tables |
| [Security](docs/security.md) | Tokens, permissions, what agents can do |
| [Troubleshooting](docs/troubleshooting.md) | Symptoms and fixes |
| [FAQ](docs/faq.md) | Short answers |
| [Development](docs/development.md) | Building, testing, the coverage gate |
| [Contributing](CONTRIBUTING.md) | How to send a change |

## Status

Version 0.0.1 ([changes](CHANGELOG.md)). kfk is young: it has run real goals end to end on macOS with a pool of
accounts, but only on small streams so far. Known gaps:

- **macOS only.** The token vault is the Keychain.
- **The web interface acts only through terminals.** Each node has a live terminal in the browser; there are no
  buttons for tasks or accounts yet. Those go through cos or the CLI.
- **Nothing restarts kfk after a reboot.** Run `kfk up` again; cos resumes its conversation.
- **Coordination is still the larger share of tokens on small tasks.** In the first pilots, eight micro-tasks all
  passed on the first attempt, at about 24k weighted tokens each for the agents, while cos and the orchestrators
  spent roughly three times that in total. Fixed coordination cost dominates tiny streams; measure your own with
  `kfk report`.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in kfk by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

kfk bundles two libraries in `crates/kfk-web/src/assets/`, each distributed under its own license:
[htmx](https://htmx.org) (`htmx.min.js`, Zero-Clause BSD) and [xterm.js](https://xtermjs.org) 6.0.0 with its fit
addon 0.11.0 (`xterm.js`, `xterm.css`, `xterm-addon-fit.js`; MIT, see `XTERM-LICENSE`).

kfk is an independent project. It is not made or endorsed by Anthropic. Claude and Claude Code are products of
Anthropic.
