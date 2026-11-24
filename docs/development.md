# Development

## Build

```sh
cargo build                  # debug: target/debug/kfk
cargo build --release        # what install.sh installs
./install.sh --yes           # build and install over the current binary, keeping your data home
```

Tested with stable Rust 1.100 on macOS. There is no build script and no code generation; the role prompts and
the web assets are compiled in with `include_str!`.

After installing a new binary, restart the services so they run it:

```sh
kfk mizan stop && kfk mizan start
kfk web stop && kfk web start
```

Nodes pick it up at their next start. Agents use whatever is installed when they are launched.

## Test

```sh
cargo test                   # everything, including the end-to-end test (about a minute)
cargo test -p kfk-store      # one crate
tests/coverage.sh            # everything, under coverage, with the gate
```

The tests need `git`, `tmux`, `sqlite3`, `curl` and `python3` on `PATH`. None of them talks to Claude or uses a
real account.

### Three layers

1. **Unit and integration tests per crate** (`crates/*/tests/`). Pure crates are tested as functions. The store
   is tested against a temporary SQLite file. The engine is tested with a fake `claude`: a shell script the test
   writes, selected with `KFK_CLAUDE`. The web crate renders every page against a temporary store and a fake
   `Probe`, through the router, without opening a socket.

2. **The end-to-end test** (`tests/e2e.sh`, run by `crates/kfk-cli/tests/e2e.rs`). It drives the built binary the
   way a user would: a throwaway data home, a private tmux server, a file vault instead of the Keychain,
   [`tests/fake-claude`](../tests/fake-claude) instead of Claude, and a throwaway PostgreSQL. It covers `up`,
   orchestrators on demand, `exec` with dependencies and escalation, rotation, restart of a dead node, idle stop,
   the log archive, the web view, and `down`/`up`.

   ```sh
   KFK_BIN=target/debug/kfk bash tests/e2e.sh       # run it alone
   KFK_PG_BIN=/opt/pg/bin bash tests/e2e.sh         # where PostgreSQL's tools are, if not on PATH
   KFK_E2E_TMP=/some/dir bash tests/e2e.sh          # where its scratch directory goes (default /tmp)
   KEEP=1 bash tests/e2e.sh                         # keep the scratch directory afterwards
   ```

   Without PostgreSQL the archive checks are skipped and say so.

3. **Pilots with real Claude.** Not automated. Before a release, run a real goal through cos in a scratch
   repository and read `kfk report`.

One Keychain test writes and deletes an entry under the service name `kfk-selftest`. It never touches the `kfk`
service that holds real tokens.

## The coverage gate

```sh
cargo install cargo-llvm-cov        # once
tests/coverage.sh
```

```
edge  kfk-mizan/src/round.rs: 41%
...
core: 1907/1907 lines | edges: 665/1106 lines (60%)
core logic: 100% of lines
```

The rule: **core logic is at 100% line coverage, and the script fails otherwise.** Core is every source file
except the process edges, which are listed by name in the script:

| Edge | Why it is not gated |
| --- | --- |
| `kfk-cli/*` | Argument parsing and printing; exercised by the end-to-end test. |
| `kfk-mizan/src/{tmux,node,node_main,round,slot,daemon}.rs` | tmux, long-lived child processes, the daemon loop. |
| `kfk-logs/src/{pg,sync,status,read}.rs` | PostgreSQL. |
| `kfk-web/src/{live,serve,ws}.rs` | The real probe, the listening socket, the terminal's WebSocket. |
| `kfk-term/src/pty.rs` | A child process on a pseudo-terminal. |
| `kfk-engine/src/detach.rs` | A closure that runs in a forked child, where coverage cannot see. |

Edges are still run by the end-to-end test and their coverage is printed, so a drop is visible. When you add logic
to an edge file, move the logic into a core file and leave only the call behind. That is how `kfk-web` keeps its
routes in core: the handlers take a `Probe`.

## Conventions

- **Small files, one concern each.** No file over roughly 200 lines. When a file gains a second concern, split it.
  When a crate gains a second concern, make a crate.
- **Pure where possible.** A function that decides something takes values and returns a value. Reading the clock,
  the environment, a file or a process happens at the edge and is passed in.
- **Tests first.** Write the test that states the behaviour, watch it fail, then make it pass. Test names are
  sentences: `a_limited_account_is_left_for_another`.
- **Unix manners.** Plain text out, one record per line, `--json` where a script wants it, errors on standard
  error as `kfk: <what happened and what to do>`, meaningful exit statuses, no output when there is nothing to say.
- **Errors tell the user the next command.** `slot 3 has no token (kfk auth 3)`.
- **No LLM in mizan.** Anything that can be a rule is a rule.
- **Nothing is deleted** from tasks, runs or events.
- **Schema changes are additive.** New tables go in `SCHEMA` (`create table if not exists`); new columns go in
  `ADDED` (`alter table ... add column`), which is applied on every open and tolerates "already there".
- `cargo fmt` is not enforced; match the file you are in. `cargo clippy` should be quiet.

## Adding a command

1. The argument shape goes in `crates/kfk-cli/src/cli.rs`.
2. The behaviour goes in a function in the crate that owns the concern, with tests there.
3. A thin function in `crates/kfk-cli/src/cmd/<group>.rs` calls it and prints; one line in `cmd/mod.rs` dispatches.
4. Add a check to `tests/e2e.sh` if it touches a process.
5. Document it in [`docs/commands.md`](commands.md).

## Adding a config key

Add it to `crates/kfk-store/src/defaults.rs` with its default. `kfk config` then knows it. Read it with
`store.cfg("key")` or `store.cfgi("key")`. Document it in [`docs/configuration.md`](configuration.md).

## Changing how the fleet behaves

The three prompts in [`roles/`](../roles) are most of the fleet's behaviour. Keep them short: every line is paid
for on every turn of every node. After a change, rebuild, install, and start nodes afresh
(`kfk down && kfk up --fresh`).

## The fake Claude

[`tests/fake-claude`](../tests/fake-claude) stands in for `claude`:

- with `-p` it is an agent: it reads the prompt, writes `out.txt` (`good`, or `wrong` on the first attempt when the
  prompt contains `FLAKY`), commits, and prints a small stream with usage and a rate-limit event;
- without `-p` it is a node: it prints its seat, slot and arguments and then sleeps.

Use it to try kfk without spending anything:

```sh
export KFK_HOME=/tmp/kfk-play KFK_CLAUDE=$PWD/tests/fake-claude KFK_TOKEN_FILE=/tmp/kfk-play/tok.json KFK_TMUX_SOCKET=kfk-play
kfk up --dir /tmp/some-git-repo
```

## Layout

```
Cargo.toml            the workspace
crates/               one crate per concern (see docs/architecture.md)
roles/                cos.md, orch.md, agent.md
tests/                e2e.sh, fake-claude, ws-type (a WebSocket client for the terminal check), coverage.sh
docs/                 this documentation
install.sh
```
