# FAQ

### What does the name mean?

*kun fayakoon* (كن فيكون) is Arabic for "be, and it is". You say what should exist; the fleet makes it so. *mizan*
(ميزان) means "balance": the layer that weighs accounts against each other.

### Do I need several Claude accounts?

No. With none added, kfk runs everything on the login Claude Code already has. A pool only matters when one
account's usage limits are too small for the work.

### Does it work with an API key instead of a subscription login?

Not as built. kfk removes `ANTHROPIC_API_KEY` from the environment of the sessions it starts and injects OAuth
tokens from `claude setup-token`. Usage tracking and rotation are written around subscription usage windows.

### Does it run on Linux or Windows?

Not yet. Two things are macOS-specific: the token vault (the Keychain, through the `security` tool) and the way
`kfk auth` records the sign-in (BSD `script`). The rest is portable Rust, tmux and git. A Linux vault (for example
the Secret Service) is the main missing piece; contributions are welcome. Windows would need WSL.

### Why tmux?

cos and the orchestrators are interactive Claude Code sessions. tmux gives each a real terminal that survives your
window closing, that you can look into (`kfk attach`), and that mizan can read and restart without any LLM.

### Why are agents separate processes instead of Claude Code's own subagents?

So that kfk, not a model, controls four things: which account each one runs on, which model and effort it gets,
whether its result counts (the check), and what it cost. A subagent started inside a session inherits that
session's account and is judged by that session.

### How is this different from just asking Claude Code to do the whole thing?

For a small change, it is not, and you should just ask. kfk is for work that is wider than one context: many
independent changes, in parallel, over hours or days, across usage limits, where you want each piece verified and
the cost visible.

### How long can it run?

The design goal is days. Tasks, attempts and decisions are in SQLite, so nothing depends on a session's memory;
nodes are restarted and resumed; agents are requeued if their runner dies. Set `capacity_wait_s` so tasks wait for
usage windows to reset instead of blocking. What is not there yet: nothing brings the fleet back after a reboot
except you running `kfk up`.

### What happens when a session's context fills up?

Nodes are started with a compaction size (`cos_compact`, `orch_compact`: 200k tokens) and compact themselves when
they reach it. That is safe because the state they need is in kfk, and their role prompt tells them to read it
after a restart. Agents are too short-lived for it to matter in most tasks.

### Why did my task use a cheaper model than I expected?

The class decides: `mech` starts at haiku, `impl` at sonnet. It escalates by itself when the check fails. Pin a
task with `--model` and `--effort`, raise its class, or change the ladder
([Configuration](configuration.md#models)).

### Can agents talk to each other, or ask me something?

No. An agent has one task and no one to ask. If it cannot finish, it stops and says why in a few lines, the check
fails, and the orchestrator reads why. Questions for you come only from cos.

### Is my code sent anywhere new?

kfk makes no network requests of its own. The Claude Code sessions it starts behave as they would if you ran them
yourself. See [Security](security.md).

### Can I use it without cos?

Yes. `kfk task add`, `kfk run` and `kfk exec` are a complete task runner with checks, retries, worktrees and
account choice. cos and orchestrators are a layer on top that writes the tasks for you.

### Can I change what cos or the orchestrators do?

Edit `roles/cos.md`, `roles/orch.md` or `roles/agent.md`, rebuild and reinstall. They are short on purpose.

### Where is everything stored? How do I back it up?

In the data home (default `~/.kfk`): `kfk.db` (SQLite), `runs/`, `wt/`, `log/` and `pg/`. `kfk status` prints the
path. See [Architecture](architecture.md#the-data-home) and [Log archive](logs.md#notes).

### How do I uninstall?

```sh
kfk down --force; kfk web stop; kfk logs stop
for s in $(seq 0 9); do kfk rm $s --yes 2>/dev/null; done     # Keychain entries of the slots you used
rm ~/.local/bin/kfk ~/.config/kfk/home
rm -rf ~/.kfk                                                  # your data home: this deletes all history
```

Task branches (`kfk/T*`) and worktrees remain in your repositories; remove them with git.

### The coordination share in `kfk report` is high. Is that bad?

On a handful of tiny tasks, yes, and expected: starting cos and an orchestrator costs a fixed amount, and small
tasks do not amortise it. Watch it on real streams. If it stays high: give cos larger goals, let streams have more
micro-tasks per orchestrator, use a cheaper model for orchestrators (`orch_model`), and keep the role prompts
short.
