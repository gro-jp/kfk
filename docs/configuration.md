# Configuration

Settings live in the kfk database, not in a file.

```sh
kfk config                       # every key, its value, and "(default ...)" where you changed it
kfk config max_agents            # one value
kfk config max_agents 4          # set it
```

Most settings are read when they are used, so a change applies to the next agent, the next mizan round or the next
node start. `web_addr` needs `kfk web stop && kfk web start`; `tmux_session` needs `kfk down && kfk up`.

## Keys

### Fleet shape

| Key | Default | Meaning |
| --- | --- | --- |
| `project_dir` | *(set by `kfk up --dir`)* | The repository nodes start in, and the default `repo` of new tasks. |
| `max_orchestrators` | `7` | How many orchestrators may be up at once. |
| `max_agents` | `8` | How many agents may run at once, across all streams. |
| `tmux_session` | `kfk` | The name of the tmux session that holds the nodes. |

### Models

Model names are passed to `claude --model` as they are; use the aliases your Claude Code version accepts
(`haiku`, `sonnet`, `opus`, `fable`) or full model ids. Efforts go to `--effort`.

| Key | Default | Meaning |
| --- | --- | --- |
| `cos_model`, `cos_effort` | `fable`, `high` | cos. |
| `orch_model`, `orch_effort` | `opus`, `high` | An orchestrator started without `--model` / `--effort`. |
| `ladder_mech` | `haiku:low sonnet:medium opus:high` | Model and effort for attempts 1, 2, 3, ... of a `mech` task. The last rung repeats. |
| `ladder_impl` | `sonnet:medium opus:high fable:high` | The same for `impl`. |
| `ladder_hard` | `opus:high fable:high fable:xhigh` | The same for `hard`. |
| `ladder_deep` | `fable:high fable:max` | The same for `deep`. |

```sh
kfk config ladder_impl "sonnet:medium sonnet:high opus:high"
```

### Attempts and time

| Key | Default | Meaning |
| --- | --- | --- |
| `max_attempts` | `3` | Attempts before a task is `failed`. A task can override it. |
| `timeout_s` | `1200` | Seconds an agent may run per attempt. It is then stopped, and the check still runs on what it left. |
| `check_timeout_s` | `900` | Seconds a check may run. Past that it counts as exit 124. |

### Context size

Sessions compact their context when it reaches this many tokens. Small windows keep each turn cheap.

| Key | Default | Meaning |
| --- | --- | --- |
| `cos_compact` | `200000` | cos. |
| `orch_compact` | `200000` | Orchestrators. |
| `agent_compact` | `150000` | Agents. |

### Accounts and limits

| Key | Default | Meaning |
| --- | --- | --- |
| `agent_max_util` | `97` | An account at or above this percentage of any usage window gets no new agent. |
| `node_max_util` | `95` | A node whose account reaches this is moved to another account. |
| `capacity_wait_s` | `0` | When no account has headroom: how long a task waits for a window to reset before it becomes `blocked`. `0` blocks at once. For runs that last days, use hours (`14400`). |

### mizan

| Key | Default | Meaning |
| --- | --- | --- |
| `mizan_every` | `30` | Seconds between rounds. |
| `orch_idle_s` | `1800` | An orchestrator with no open task for this long is stopped. |

### Flags passed to Claude Code

| Key | Default | Meaning |
| --- | --- | --- |
| `node_flags` | `--dangerously-skip-permissions` | Extra flags for cos and orchestrators. |
| `agent_flags` | `--dangerously-skip-permissions` | Extra flags for agents. |

With the defaults, sessions run tools without asking. Read [Security](security.md) before you use kfk on anything
you care about, and narrow these if you need to, for example:

```sh
kfk config agent_flags "--permission-mode acceptEdits --allowedTools Bash(cargo:*) Bash(git:*)"
```

`claude --help` lists the flags your version has. An agent that stops to ask for a permission nobody can grant
will run into its time limit, so make sure the flags allow what the task needs.

### Services

| Key | Default | Meaning |
| --- | --- | --- |
| `web_addr` | `127.0.0.1:7878` | Where the web view listens. |
| `pg_bin` | *(empty: use `PATH`)* | The directory with `initdb`, `pg_ctl` and `postgres`. |
| `pg_port` | `54329` | Names the archive's Unix socket; no TCP port is opened. |
| `claude_projects` | *(empty: `~/.claude/projects`)* | Where Claude Code keeps session transcripts. |

## Environment variables

| Variable | Meaning |
| --- | --- |
| `KFK_HOME` | The data home, for this command or shell. Overrides what the installer recorded. |
| `KFK_CLAUDE` | The Claude Code binary to run. Default `claude` on `PATH`. |
| `XDG_CONFIG_HOME` | Where the data-home pointer lives: `$XDG_CONFIG_HOME/kfk/home`. Default `~/.config`. |
| `KFK_TOKEN_FILE` | **Tests only.** Keeps tokens in this JSON file, in clear text, instead of the Keychain. |
| `KFK_TMUX_SOCKET` | **Tests only.** Uses a separate tmux server with this socket name. |

Where the data home comes from, in order: `$KFK_HOME`; the path in `~/.config/kfk/home`; `~/.kfk`.

### What kfk sets for the sessions it starts

| Variable | Set for | Value |
| --- | --- | --- |
| `CLAUDE_CODE_OAUTH_TOKEN` | Nodes and agents on a pool account | The account's token. |
| `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | Nodes and agents | The compaction size above. |
| `KFK_SEAT`, `KFK_ROLE`, `KFK_ACCOUNT` | Nodes | The seat name (`cos`, `o1`), `cos` or `orch`, the account label. |
| `KFK_SLOT` | Nodes and agents | The slot number (`-1` for the saved login). |
| `KFK_TASK` | Agents and checks | The task id. |
| `KFK_DIR` | Checks | The directory the check runs in. |
| `KFK_HOME` | Nodes | The data home, so `kfk` commands run by a node reach the same store. |

Sessions do not inherit `CLAUDE_CODE_*`, `ANTHROPIC_API_KEY` or `ANTHROPIC_AUTH_TOKEN` from the shell that started
kfk.

## Several fleets on one machine

Give each its own data home and tmux session:

```sh
export KFK_HOME=~/kfk-work
kfk config tmux_session kfk-work
kfk config web_addr 127.0.0.1:7879
kfk up --dir ~/code/work-repo
```

Tokens in the Keychain are shared between them (service `kfk`, account `slot<N>`); the account pools and usage
figures are per data home. Running two fleets at once is untested: node names (`cos`, `o1`) are the same in both,
and the messages between sessions are addressed by name.
