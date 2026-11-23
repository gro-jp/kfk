# Troubleshooting

Start here:

```sh
kfk status          # nodes, queues, agents, anything waiting on you
kfk mizan           # is mizan running; its last actions
kfk task show <id>  # why a task failed
```

Logs are in `<data home>/log/`: `mizan.log`, `web.log`, `pg.log`, `T<id>.log` (a task's runner) and
`exec-T<id>.log`.

## Starting

**`kfk: command not found`**
The installer put the binary in `~/.local/bin`. Add it to your `PATH`.

**`kfk up` says cos is waiting for the owner**
Claude Code is showing a dialog: folder trust, or the first-use warning for bypass-permissions mode. Run
`kfk attach`, answer it, detach with `Ctrl-b d`. This happens once per folder.

**`tmux could not start cos`**
Is `tmux` installed and can it start a server? Try `tmux new -d -s test && tmux kill-session -t test`.

**`cannot start claude`**
Claude Code is not on the `PATH` kfk sees. Either fix `PATH` or `export KFK_CLAUDE=/full/path/to/claude`.

**cos keeps restarting**
`kfk mizan` shows the restarts; `kfk attach` shows the node's screen and its exit line. mizan gives up after three
restarts in an hour. The usual causes are an account that needs a new sign-in (`kfk check`) and a session that can
no longer be resumed (`kfk down && kfk up --fresh`).

**kfk uses the wrong data home**
`kfk status` prints `data: <path>` on its first line. The order is `$KFK_HOME`, then `~/.config/kfk/home`, then
`~/.kfk`. Run `./install.sh --home <path>` to change what is recorded.

## Tasks

**`T7 lacks a spec, a definition of done or a check command`**
All three are required before a task can run: `kfk task set 7 spec=... dod=... check='...'`.

**A task failed**
`kfk task show 7` prints the tail of the failing check and the agent's last words. Open
`http://127.0.0.1:7878/task/7` for the full transcript of each attempt. Usually the spec left something out or the
check tests something the spec did not ask for. Fix it, then `kfk run 7`.

**A task is `blocked`: "no account has headroom"**
Every account is at its limit. The message carries the earliest reset. Then `kfk run 7`, or `kfk task reopen 7`
and `kfk exec <stream>`. To make runners wait instead of blocking: `kfk config capacity_wait_s 14400`.

**A task is `blocked`: "kfk could not run it"**
The message says why: most often the `claude` binary was not found, or `git worktree add` failed (the branch
`kfk/T7` is checked out somewhere else, or the `base` does not exist).

**An agent was "killed at its 1200s limit"**
The check still ran on what it left. If the task is honestly bigger, split it, or raise its limit:
`kfk task set 7 timeout=2400`.

**The check passes by hand but fails under kfk**
The check runs with `bash -c` in the task's worktree (`<data home>/wt/T7`), not in your checkout, and with a clean
shell. Use paths relative to the repository root; do not rely on files that are not committed (a local
virtualenv, `node_modules`, a `.env`). Reproduce it exactly:

```sh
cd ~/.kfk/wt/T7 && bash -c "$(kfk task show 7 --json | jq -r .chk)"; echo "exit $?"
```

**A task stays `running` but nothing is happening**
Its runner died (a reboot, a kill). mizan requeues it within one round; without mizan, `kfk status` does the same
check.

**`kfk exec` says a child is "waiting on a dependency that did not finish"**
The task it depends on failed or is blocked. Fix that one first.

**`git worktree add failed`**
A stale worktree or branch from an earlier task with the same id (for example after you deleted the data home but
not the branches). `git -C <repo> worktree prune`, and delete the leftover `kfk/T<id>` branch if you do not need it.

## Accounts

**`kfk auth`: "no token found in the transcript"**
The sign-in did not finish, or the terminal was resized while it ran. Run it again in a normal-width window and do
not resize.

**`kfk auth`: "the server rejected this token"**
The token was cut or belongs to an account that cannot use it. Sign in again.

**`slot N did not read back what was written (Keychain locked?)`**
Unlock the login keychain (`security unlock-keychain`) and repeat. Over SSH the keychain is usually locked.

**`NO TOKEN` in `kfk accounts`**
The Keychain entry is gone. `kfk auth <slot>`.

**Usage shows `?` or looks old**
kfk only learns usage when the account is used. `kfk check` refreshes every account with one small call each.

**A node did not move off a full account**
mizan needs another account under 80% (15 points below `node_max_util`), and moves a node at most three times an
hour. `kfk swap <seat> <slot>` moves it by hand.

## Web

**"the web interface did not start"**
The address is in use. `kfk config web_addr 127.0.0.1:7900`, then `kfk web start`. The reason is in
`<data home>/log/web.log`.

**A transcript page says the file is "neither on disk nor in the log archive"**
Claude Code deleted the file before it was archived. Set up the archive ([Log archive](logs.md)) so this cannot
happen again.

## Log archive

**`cannot run initdb`**
PostgreSQL's tools are not found. `kfk config pg_bin <directory that contains initdb>`.

**`PostgreSQL did not start`**
Read `<data home>/log/pg.log`. Common causes: the data home path is too long for a Unix socket (keep it under
about 90 characters), or a server from a previous run is still shutting down.

## Starting over

```sh
kfk down --force && kfk web stop && kfk logs stop
```

Your tasks, runs and history are the directory printed by `kfk status` as `data:`. Moving that directory away
gives you an empty kfk; tokens stay in the Keychain (`kfk rm <slot>` removes them). Task branches (`kfk/T*`) and
worktrees stay in your repositories until you delete them.
