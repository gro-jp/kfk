# Web interface

A view of the fleet: what needs you, what is running, what is stuck, what was done, and every transcript. The
pages only show; the one way to act is the [live terminal](#terminals) on each node.
It is one process inside the `kfk` binary (Axum, Maud and HTMX); there is nothing else to install and no build
step.

```sh
kfk web start       # also started by `kfk up`
kfk web             # is it running, and where
kfk web stop
kfk web run         # in the foreground, for debugging
```

Default address: <http://127.0.0.1:7878>. Change it with:

```sh
kfk config web_addr 127.0.0.1:9000
kfk web stop && kfk web start
```

`kfk down` leaves the web view running, so history stays browsable while the fleet is down.

> The web view has **no login**. It listens on localhost only by default. If you bind it to another address, anyone
> who can reach that address can read every task, prompt and transcript (terminals turn themselves off there). Put
> it behind something that authenticates, or use an SSH tunnel: `ssh -L 7878:127.0.0.1:7878 your-machine`.

## Pages

### Overview `/`

Refreshes itself every 4 seconds.

- **Needs you.** Streams blocked on a decision that only you can make, and nodes waiting on a dialog. When this is
  empty, nothing is waiting for you.
- **Nodes.** cos and each orchestrator: state, model, account, how full that account is, how full the node's
  context is before it compacts, and its queue.
- **Agents in flight.** Each running attempt: task, model, account, how long.
- **Streams**, in six lanes:

  | Lane | Meaning |
  | --- | --- |
  | Needs you | Waiting for a decision only you can make |
  | Stuck | Failed or blocked; an orchestrator or cos has to act |
  | In flight | Agents are working on it now |
  | Queued | Cut into tasks, waiting for a free agent or a dependency |
  | Planned | Handed to an orchestrator, not yet cut into tasks |
  | Done | Its check passed |

- **Last 24 hours.** Tasks passed, weighted tokens, tokens per passed task, and the coordination share.

### A task `/task/<id>`

The brief (spec, done-when, check), its children with their state, every attempt, the tail of the last failing
check, the agent's last words, and the task's history including the decisions noted on it. A stream's page lists
its micro-tasks; click through to each. Refreshes every 5 seconds.

### An attempt `/run/<id>`

One agent run: outcome, model, account, duration, tokens, the exact prompt the agent started from, the check's
output, and the full transcript.

### Nodes `/nodes` and `/node/<seat>`

For one node: its **terminal**, the open work it owns, its tokens by hour over the last day, and every session it
has had.

## Terminals

Every node has a live terminal: on its page (`/node/<seat>`), and alone on a page of its own (`/shell/<seat>`,
linked as **Open** wherever nodes are listed). It is the node's real session, the same one `kfk attach` shows: what
you type goes to it, and what it prints comes back.

- **Full screen** fills the window with the terminal. In Chrome and Edge it also goes to real full screen with
  Escape still reaching the node. In Safari and Firefox it fills the window only, because there Escape would leave
  full screen instead of interrupting Claude.
- **Scrolling:** the mouse wheel scrolls back through the node's history. Scroll back down to the bottom, or press
  `q`, to type again.
- **Selecting text:** hold Option (macOS) or Shift while dragging. Paste works as usual.
- **Several viewers** can have the same node open, in browsers and in `kfk attach`; they see the same screen, at
  the size of whichever viewer was active last. A web terminal does not change which window `kfk attach` shows,
  nor its mouse or status-line settings.
- **One terminal, one node.** If mizan restarts or moves the node, its terminal disconnects instead of showing
  another node; **Connect** opens the new one.
- Closing the page or losing the connection leaves the node running. **Connect** opens the terminal again.

Who may type:

- Terminals work only when the web view listens on this machine (`web_addr` on `127.0.0.1`, `localhost` or `::1`).
  On any other address the node page shows the last lines of the screen instead, refreshed every 3 seconds.
- The server refuses a terminal to a page from any other site, and to a request that names another host, so a web
  page you happen to have open cannot type into a node.
- `kfk config web_shell off` turns terminals off; `on` (the default) brings them back. It applies at once.
- To use a terminal from another computer, tunnel to the loopback address:
  `ssh -L 7878:127.0.0.1:7878 your-machine`, then open <http://127.0.0.1:7878>.

There is no password. Any program running as any user on the machine can reach a loopback address; on a machine you
share with people you do not trust, turn terminals off. See [Security](security.md#the-web-view).

### A session `/session/<id>`

The full transcript of one node session.

### Accounts `/accounts`

The pool: each account's usage windows as bars, when they reset, and who is on it.

### History `/history`

Numbers for the last 1, 7, 30 or 365 days (`/history?days=7`): node spend, agent runs by class and model with
pass rates, the streams touched, and the event log (starts, stops, rotations, swaps, notes, results).

The event log covers everything ever recorded, newest first, 50 events to a page (`/history?days=7&page=2`). The
period applies to the numbers above it, not to the event log.

## Transcripts

Transcripts are shown as a conversation: prompts, Claude's text, tool calls, results. Long or noisy entries
(thinking, tool results, system lines) are folded; click to open. They are paged, 150 lines per page, newest page
first. "Show bookkeeping lines too" adds the lines Claude Code writes for its own use.

A transcript is read from disk while its file exists and from the [log archive](logs.md) after that, so history
outlives Claude Code's own clean-up.

## Accessibility

- Works without JavaScript; scripting only adds the periodic refresh.
- Semantic landmarks, a skip link, table headers with scope, labelled meters.
- State is never shown by colour alone: every badge carries a word and a mark.
- Follows the system light or dark setting, and reduced-motion.
- Keyboard focus is always visible.

## Machine-readable

There is no JSON API in the web process. For scripts, use the CLI:

```sh
kfk status --json
kfk task ls --json
kfk task show 7 --json
```

A liveness probe: `GET /healthz` returns `ok`.
