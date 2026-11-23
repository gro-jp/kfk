# mizan

mizan (Arabic: the balance) is kfk's base layer. It contains no LLM and makes no model calls. Everything it does is
a rule you can read in [`crates/kfk-mizan`](../crates/kfk-mizan).

It has two parts:

- **library code** that every `kfk` command uses to start, stop and move nodes;
- **a background process** that runs one *round* every `mizan_every` seconds (default 30).

```sh
kfk mizan           # is it running, and what it did recently
kfk mizan start
kfk mizan stop
kfk mizan once      # run one round now, in the foreground, and print what it did
```

`kfk up` starts it and `kfk down` stops it. Its log is `<data home>/log/mizan.log`.

## What a round does

1. **Requeue orphans.** A task marked `running` whose runner process is gone goes back to `todo`; its run is marked
   `lost`; an agent process still alive from it is stopped. The attempt is not counted.

2. For each node that should be up:

   | It sees | It does |
   | --- | --- |
   | The process or the tmux window is gone | Starts the node again with `claude --resume <session>`, so the conversation continues. No more than 3 restarts per node per hour; the third starts a fresh session in case the old one is what keeps failing. A node gets 20 seconds to come up before it can be judged dead. |
   | An orchestrator has owned no open task for `orch_idle_s` (30 minutes) | Stops it. cos starts a new one when work arrives. |
   | The pane shows a dialog only the owner can answer (folder trust, first use of bypass-permissions mode, a sign-in prompt) | Records a note on the node. `kfk status` and the web view show it under "Needs you". mizan never answers such a dialog. |
   | The node's account is at or above `node_max_util` (95%) of a usage window, or the pane shows a limit message | Picks another account that is at least 15 points under that limit, and restarts the node on it with its session kept. Recorded as a `rotate` event. |

3. **Account for node spend.** For every open node session it reads the lines its transcript gained since last
   round, and adds input, cache-write, cache-read and output tokens into hourly totals. This is what `kfk report`
   shows as "nodes".

4. **Archive logs.** If the log archive is set up, it copies new transcript lines into PostgreSQL.

A round with nothing to do costs a few SQLite reads and one `tmux capture-pane` per live node.

## Starting a node

Whoever starts a node (`kfk up`, `kfk orch start`, a restart, a rotation), the steps are the same:

1. **Choose the account**, unless the node already has one it can use: the account with the most headroom
   ([how](accounts.md#how-an-account-is-chosen)).
2. **Open a tmux window** named after the seat, in the project directory, running `kfk _node <seat>`.
3. Inside it, kfk starts `claude` with:
   - the model and effort of the seat;
   - the role prompt (`roles/cos.md` or `roles/orch.md`) appended to the system prompt;
   - a settings file that installs kfk's status line and lets the session receive messages from other sessions;
   - `--session-id <new id>` for a fresh session, or `--resume <id>` and a one-line "you were restarted" prompt;
   - a clean environment: nothing from the caller's Claude session, the account's token, and
     `CLAUDE_CODE_AUTO_COMPACT_WINDOW` set to the node's compaction size.

## The status line

Each node's status line is `kfk _statusline`. Claude Code calls it with a JSON description of the session after
each turn. kfk uses it to:

- store the account's usage windows;
- store the node's current context size;
- print a short line: `cos | Fable 5.2 | ctx 51k | wk 12%`.

This is how kfk knows usage and context size without spending a token.

## Orchestrators on demand

There is no standing pool of orchestrators. cos runs:

```sh
kfk orch start --model sonnet --effort high
```

kfk takes the lowest free name (`o1`..`o7`), gives it a fresh session and an account, starts it, and prints the
name. The limit is `max_orchestrators` (7). An orchestrator that finishes its work is stopped by mizan after the
idle time, or by hand:

```sh
kfk orch ls
kfk orch stop o2      # refused while it still owns open tasks
```

## What mizan does not do

- It does not decide what to work on, cut streams, or judge results. Those belong to cos, the orchestrators and
  the checks.
- It does not answer dialogs, accept trust prompts or sign in.
- It does not restart itself, and nothing starts it at boot. After a reboot, run `kfk up`.
