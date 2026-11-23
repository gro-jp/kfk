# Concepts

## The three kinds of session

| | How many | Lives | Does | Never does |
| --- | --- | --- | --- | --- |
| **cos** (chief of staff) | 1 | As long as the fleet is up; its conversation is resumed across restarts | Talks to you. Turns goals into streams. Starts orchestrators. Reports results. | Implement. Launch agents. |
| **orchestrator** (`o1`..`o7`) | 0 to 7, started on demand | While it owns open work; stopped after 30 idle minutes | Cuts one stream into micro-tasks, runs them with `kfk exec`, integrates the result. | Implement micro-tasks itself. Talk to you. |
| **agent** | Up to 8 at once | One attempt at one task: seconds to minutes | Exactly the task in its prompt, then exits. | Ask, report, or remember. |

cos and orchestrators are **nodes**: interactive Claude Code sessions in tmux windows. Agents are headless
`claude -p` processes.

Their instructions are three short files in [`roles/`](../roles): `cos.md`, `orch.md` and `agent.md`. They are
compiled into the binary; edit them and rebuild to change how the fleet behaves.

## Streams and tasks

Everything kfk does is a **task**, identified as `T<id>`.

- A **stream** is a task with no parent. It is one deliverable: a feature, a fix, a pull request. An orchestrator
  owns it.
- A **micro-task** is a child of a stream. It is one independently verifiable change, sized for 5 to 20 minutes of
  agent time.

A task has:

| Field | Meaning |
| --- | --- |
| `title` | One line. |
| `spec` | What to do and why, with the files, interfaces and constraints an agent with no memory needs. |
| `dod` | "Done when": the observable end state. |
| `check` | A shell command. Exit status 0 means done. |
| `class` | `mech`, `impl`, `hard` or `deep`: how difficult it is. Decides the model. |
| `repo`, `base` | The git repository, and the branch or commit work starts from. |
| `deps` | Tasks that must be done first. |
| `owner` | The orchestrator that owns it. |

A task without a spec, a definition of done **and** a check cannot be run. That rule is enforced by the store, not
by convention.

### Status

```
todo ──▶ running ──▶ done
            │
            ├──▶ failed     the check still fails after every attempt
            └──▶ blocked    kfk could not run it (no account with headroom, no binary), or someone marked
                            it as waiting on something outside kfk
cancelled                   dropped; its open children are cancelled with it
```

The web view groups streams into lanes derived from these: **Needs you** (blocked with a result that starts
`needs owner:`), **Stuck** (failed or blocked), **In flight**, **Queued**, **Planned** (not yet cut into
micro-tasks) and **Done**.

## The check

The check is the only judge.

- kfk runs it itself, in the task's worktree, after the agent has exited. The agent's own claim of success is not
  consulted.
- A check should fail before the change and pass after it: a test, a build, a `grep`, a script.
- `kfk task done <id>` runs the task's check too, and refuses to mark it done on a non-zero exit.
- A check has its own time limit (`check_timeout_s`, 15 minutes). Running past it counts as exit 124.

## Attempts, classes and the ladder

Each try at a task is an **attempt**, recorded as a **run**. The model and reasoning effort of an attempt come from
the task's class:

| Class | For | Attempt 1 | Attempt 2 | Attempt 3 |
| --- | --- | --- | --- | --- |
| `mech` | Mechanical edits, renames, config | haiku / low | sonnet / medium | opus / high |
| `impl` | A well-specified change (the default) | sonnet / medium | opus / high | fable / high |
| `hard` | Cross-module changes, tricky debugging | opus / high | fable / high | fable / xhigh |
| `deep` | Design-level work, unknown root causes | fable / high | fable / max | fable / max |

When a check fails, the next attempt climbs one rung. It works in the same worktree and is shown the tail of the
failing check, so it fixes rather than starts over. After `max_attempts` (3) the task is `failed`.

The ladders are configuration (`ladder_mech` and so on); a task can also pin `--model` and `--effort`.

An attempt that hits a usage limit or an auth error is not counted. kfk marks that account as unavailable for a
while and repeats the attempt on another.

## Worktrees

For a task in a git repository, kfk creates a worktree at `<data home>/wt/T<id>` on a new branch `kfk/T<id>`, cut
from the task's `base` (default: the repository's `HEAD`). Agents commit there. Parallel agents never share a
working directory. The orchestrator integrates the task branches the way the stream's spec says: merge, rebase,
pull request.

A task whose `repo` is a plain directory runs in that directory. A task with no repo runs in
`<data home>/work/T<id>`.

## Accounts, slots and seats

- An **account** is a Claude account you signed in with `kfk auth`.
- A **slot** is its number in the pool (0, 1, 2, ...). The slot named `login` is the login Claude Code itself has
  saved; it is used when the pool is empty.
- A **seat** is a node's place: `cos`, `o1`, ... Each seat sits on one slot at a time.

Each account has **usage windows** (a five-hour window, a weekly window, sometimes per-model windows). kfk learns
their utilisation from what the sessions themselves report as they work. It makes no calls of its own to ask.

## mizan

*mizan* (Arabic: the balance) is the layer with no LLM. Every 30 seconds it:

- puts tasks whose runner died back in the queue;
- restarts a node whose process or window is gone, resuming its session;
- stops an orchestrator that has owned no open task for 30 minutes;
- moves a node to another account when its account passes 95% of a usage window, keeping its session;
- notes when a node is stuck on a dialog only you can answer;
- adds up the tokens each node spent, from its transcript;
- copies new transcript lines into the log archive.

It also decides, at the moment an agent starts, which account that agent uses. See [mizan](mizan.md).

## Messages

Sessions talk through Claude Code's own cross-session messages, and the protocol is deliberately tiny:

| From | To | Message |
| --- | --- | --- |
| cos | orchestrator | `T7` |
| orchestrator | cos | `T7 done`, `T7 failed: <why>` or `T7 ask: <question>` |

Everything else is in the store, where it costs nothing to keep and nothing to re-read.

## Weighted tokens

kfk reports cost in **weighted tokens**: input-token equivalents.

```
weighted = input + 1.25 × cache write + 0.1 × cache read + 5 × output
```

`kfk report` splits them into **agents** (the work) and **nodes** (cos and orchestrators: the coordination), and
gives the coordination share. That one number tells you whether the fleet is delivering or talking.
