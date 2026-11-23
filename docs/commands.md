# Commands

`kfk -h` lists every command; `kfk <command> -h` shows its flags. Task ids are accepted as `7` or `T7`.

Output is plain text in aligned columns, made to be piped. Commands that list things have `--json` where a script
would want it. A closed pipe (`kfk task ls | head -3`) ends kfk quietly.

## Exit status

| Status | Meaning |
| --- | --- |
| 0 | Success. For `run`, `wait`, `exec`: the task or stream is done. |
| 1 | An error, printed as `kfk: <what and what to do>`. For `check`: at least one account is not ok. |
| 2 | Bad command line. |
| 3 | The work settled but is not done: a task failed or is blocked, or `task done` was refused by the check. |

## Fleet

| Command | Does |
| --- | --- |
| `kfk up [--dir <repo>] [--fresh]` | Starts cos, mizan and the web view. `--dir` sets the project directory (remembered). cos resumes its last conversation unless `--fresh`. Orchestrators that were up are started again. |
| `kfk down [--force]` | Stops mizan and every node. Refuses while agents are running unless `--force`; agents then finish on their own. Tasks, accounts and history are untouched. The web view keeps running. |
| `kfk attach` | Opens the tmux session on cos. Detach with `Ctrl-b d`. Other nodes are other windows: `Ctrl-b n`. |
| `kfk status [--json]` | Fleet state, each node with its account, usage, context and queue, running agents, blocked tasks, the last 24 hours. |
| `kfk report [--days <n>]` | Tasks passed, weighted tokens for agents and for nodes, tokens per passed task, coordination share, per-node spend, pass rate by class and model, streams. Default 1 day. |

## Orchestrators

| Command | Does |
| --- | --- |
| `kfk orch start [--model <m>] [--effort <e>]` | Starts an orchestrator with a fresh session on the best account and prints its name (`o1`..). Normally cos runs this. Fails when `max_orchestrators` are up. |
| `kfk orch stop <name>` | Stops it. Refused while it owns open tasks. |
| `kfk orch ls` | Nodes and their accounts. |

## Tasks

| Command | Does |
| --- | --- |
| `kfk task` | Open tasks. |
| `kfk task add "<title>" [flags]` | Adds a stream or a micro-task; prints `T<id>`. Flags below. |
| `kfk task ls [--owner <o>] [--parent <id>] [--status <s>] [--open] [--streams] [--limit <n>] [--json]` | Lists tasks, oldest first, the newest `--limit` (60). |
| `kfk task show <id> [--json]` | The task, its children by status, every attempt, the tail of the last failing check and of the agent's last words, and its notes. |
| `kfk task set <id> key=value ...` | Changes fields: `title spec dod check repo base class model effort timeout max_attempts owner deps`. A value may be `@file`. |
| `kfk task note <id> "<text>"` | Records a decision or a fact on the task. |
| `kfk task done <id> [--result "<text>"] [--force]` | Runs the task's check; marks it done only on exit 0. Refuses while children are open unless `--force`. |
| `kfk task fail <id> [--result "<why>"]` | Marks it failed. |
| `kfk task block <id> [--result "<why>"]` | Waiting on something outside kfk. A result that starts `needs owner:` puts it under "Needs you". |
| `kfk task cancel <id> [--result "<why>"]` | Drops it and its open children. |
| `kfk task reopen <id>` | Back to `todo`, attempts reset. |

`kfk task add` flags:

| Flag | Meaning |
| --- | --- |
| `--parent <id>` | The stream this micro-task belongs to. Without it the task is a stream. |
| `--owner <seat>` | The orchestrator that owns it. Default: the parent's owner, else the calling node. |
| `--spec <text \| - \| @file>` | What and why. |
| `--dod <text \| - \| @file>` | The observable end state. |
| `--check '<command>'` | Exit status 0 means done. |
| `--class mech\|impl\|hard\|deep` | Default `impl`. |
| `--repo <path>` | Default: the parent's repository, else the project directory. |
| `--base <ref>` | Where the task's branch starts. Default: the repository's `HEAD`. |
| `--dep <id>` | A task that must be done first. Repeatable. |
| `--model <m>`, `--effort <e>` | Pin them instead of using the class ladder. |
| `--timeout <seconds>` | Per attempt. Default `timeout_s`. |
| `--max-attempts <n>` | Default `max_attempts`. |

## Running

| Command | Does |
| --- | --- |
| `kfk run <id> [--detach]` | Runs one task until it settles: an agent per attempt, the check after each, escalation on failure. |
| `kfk exec <id> [--jobs <n>] [--detach]` | Runs every `todo` child of a stream in dependency order, in parallel, and prints one summary. Refuses if a child lacks a spec, a definition of done or a check. |
| `kfk wait <id>` | Blocks until a task settles, then prints its attempts and outcome. |

## Accounts and seats

| Command | Does |
| --- | --- |
| `kfk auth <slots> [--paste] [--from-log <file>] [--label <email>]` | Signs in an account and stores its token. `<slots>` is one slot, a list (`3,12`) or a range (`10-20`). |
| `kfk accounts` (alias `kfk ls`) | The pool with usage windows and who is on each account. |
| `kfk check [<slots>]` | One small call per account: `ok`, `REJECTED`, or a non-auth failure. Refreshes usage windows. |
| `kfk off <slot>` / `kfk on <slot>` | Takes an account out of the choice, or puts it back. |
| `kfk rm <slot> [--yes]` | Removes the account and its Keychain entry; nodes on it are moved. |
| `kfk swap <seat> [<slot>] [--no-restart]` | Moves a node to another account (default: the best one). A live node is restarted with its session kept. |
| `kfk seat ls` | Nodes and their accounts. |
| `kfk seat set <seat> <slot>` | Puts a seat on a slot at its next start. |
| `kfk as <slot \| seat> <command...>` | Runs a command with that account's token in its environment. |

## Services

| Command | Does |
| --- | --- |
| `kfk mizan [start \| stop \| status \| once]` | The zero-LLM layer. `once` runs a single round and prints what it did. |
| `kfk web [start \| stop \| status \| run]` | The web view. `run` stays in the foreground. |
| `kfk logs [init \| sync \| status \| stop]` | The PostgreSQL log archive. |
| `kfk logs cat <path>` | Prints an archived file to standard output. |

## Configuration

| Command | Does |
| --- | --- |
| `kfk config` | Every key, its value, and the default where it differs. |
| `kfk config <key>` | One value. |
| `kfk config <key> <value>` | Sets it. See [Configuration](configuration.md). |

## Recipes

```sh
# what is waiting on me?
kfk status --json | jq -r '.blocked[] | "T\(.id) \(.result)"'

# every task that needed more than one attempt this week
kfk task ls --limit 500 --json | jq -r '.[] | select(.attempts > 1) | "T\(.id) \(.class) \(.attempts) \(.title)"'

# run a stream and get notified
kfk exec 12; osascript -e 'display notification "stream 12 settled" with title "kfk"'

# follow one agent while it works
tail -f ~/.kfk/runs/<run id>/stream.jsonl | jq -r 'select(.type=="assistant") | .message.content[]? | .text // .name'

# the cost of one stream
kfk task ls --parent 12 --json | jq '[.[].id]'      # then: kfk task show <id> for each
```
