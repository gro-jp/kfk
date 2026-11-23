# Getting started

This page takes you from nothing to a finished, checked task. It assumes macOS.

## 1. Prerequisites

```sh
claude --version      # Claude Code CLI; run `claude` once and sign in if you have not
cargo --version       # Rust, from https://rustup.rs
git --version
tmux -V
```

PostgreSQL is optional. Without it everything works except the permanent log archive
([Log archive](logs.md)).

## 2. Install

```sh
git clone https://github.com/gro-jp/kfk.git kfk && cd kfk
./install.sh
```

The installer asks:

| Question | Default | Meaning |
| --- | --- | --- |
| Data home | `~/.kfk` | Where kfk keeps its SQLite database, agent runs, worktrees, logs and the PostgreSQL cluster. Put it on a fast disk with room to grow. |
| PostgreSQL bin directory | wherever `initdb` is on your `PATH` | Used only by `kfk logs`. Empty means "later". |

Flags do the same without questions:

```sh
./install.sh --prefix ~/.local --home /data/kfk --pg-bin /opt/homebrew/opt/postgresql@17/bin --yes
```

What it does:

1. Builds `target/release/kfk`.
2. Copies it to `<prefix>/bin/kfk` (default `~/.local/bin/kfk`). Make sure that directory is on your `PATH`.
3. Records the data home in `~/.config/kfk/home` (or `$XDG_CONFIG_HOME/kfk/home`).
4. Stores the PostgreSQL directory with `kfk config pg_bin`.

Run it again at any time to upgrade or to change the answers. The environment variable `KFK_HOME` overrides the
recorded data home for one command or one shell.

## 3. Start the fleet

```sh
kfk up --dir ~/code/my-repo
```

`--dir` is the repository cos and its orchestrators work in. kfk remembers it; later a plain `kfk up` is enough.

`kfk up` starts three things:

- **cos**, in a tmux session named `kfk`;
- **mizan**, the background process that looks after sessions and accounts;
- the **web view** at <http://127.0.0.1:7878>.

The first time Claude Code is started in a folder it asks whether you trust the folder. kfk never answers that for
you. `kfk up` and `kfk status` tell you when cos is waiting on it:

```
cos: waiting for the owner: accept the folder trust dialog (kfk attach)
```

## 4. Talk to cos

```sh
kfk attach
```

You are now in the tmux session, in front of cos. Detach with `Ctrl-b d`; the fleet keeps running.

Give cos a goal. Good goals name the outcome and how you will know it is there:

> In this repo add a module `textkit/stats.py` with mean, median and mode for a list of numbers, stdlib only,
> ValueError on empty input, with pytest tests. Deliver on a local branch `stats`; no push.

What happens next, without you:

1. cos writes the stream into kfk with a spec, a definition of done and a check command.
2. cos starts an orchestrator (`o1`) and sends it one line: `T2`.
3. The orchestrator reads `T2`, cuts it into micro-tasks, each with its own check, and runs `kfk exec 2`.
4. kfk starts one agent per micro-task, in parallel where the dependencies allow, each in a worktree on branch
   `kfk/T<id>`. When an agent exits, kfk runs the task's check.
5. The orchestrator merges the task branches as the spec says and marks the stream done. kfk runs the stream's own
   check first and refuses if it fails.
6. cos tells you, in three lines or fewer, what was delivered.

If cos needs a decision that only you can make, it asks you a question with options, and the stream appears under
**Needs you** in the web view.

## 5. Watch

```sh
kfk status
```

```
fleet: up | mizan: on | project: /Users/you/code/my-repo | data: /Users/you/.kfk
seat  node  slot   usage  model        ctx  running  todo  blocked
cos   up    login  12%    fable/high   51k  0        0     0
o1    up    login  12%    sonnet/high  38k  2        1     0

agents running: 2 of 8
  T4 run 3 | haiku/low slot login | 9s | stats.mean + tests
  T5 run 4 | sonnet/medium slot login | 9s | stats.median + tests
```

```sh
kfk task ls              # every open task
kfk task show 4          # one task: brief, attempts, the tail of a failing check
kfk report               # what passed, what it cost, how much was coordination
```

Or open <http://127.0.0.1:7878>. Every stream, task, attempt and transcript is one click deep.

## 6. A task by hand

cos and orchestrators are optional. The engine underneath is a queue of tasks with checks:

```sh
cd ~/code/my-repo
kfk task add "fizzbuzz script" --class mech --repo . \
  --spec "Write fizzbuzz.py: prints 1..100, Fizz for multiples of 3, Buzz for 5, FizzBuzz for both." \
  --dod  "python3 fizzbuzz.py prints 100 lines; line 15 is FizzBuzz" \
  --check 'test "$(python3 fizzbuzz.py | wc -l)" -eq 100 && test "$(python3 fizzbuzz.py | sed -n 15p)" = FizzBuzz'
```

```
T1
```

```sh
kfk run 1
```

```
  run 1 try 1: PASS | haiku/low slot login | 0m08s | 2 turns | out 1k, weighted 24k | check passed
T1 DONE | branch kfk/T1, check passed on attempt 1 (haiku/low, slot login)
```

The work is committed on branch `kfk/T1` in your repository. Merge it when you like.

## 7. Add accounts (optional)

```sh
kfk auth 0
```

A browser window opens. Sign in as the account you want in slot 0, and type its email when asked for a label. Repeat
with `kfk auth 1`, `kfk auth 2`, and so on. From then on mizan spreads sessions and agents over the pool. Details:
[Accounts](accounts.md).

## 8. Stop

```sh
kfk down         # stops mizan and every node; tasks, accounts and history are untouched
kfk web stop     # the web view is a separate service and keeps running until you stop it
```

Next time, `kfk up` resumes cos's conversation where it left off. `kfk up --fresh` gives cos a new one.

## Where to go next

- [Concepts](concepts.md) for the model behind all this.
- [Tasks and checks](tasks-and-checks.md) for writing tasks that pass on the first attempt.
- [Configuration](configuration.md) to change models, limits and timeouts.
