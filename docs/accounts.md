# Accounts

kfk can run on a single Claude login or on a pool of accounts. With a pool, every session and every agent is put on
the account with the most headroom, and long-lived sessions are moved before they hit a limit.

Make sure the way you use several accounts is allowed by the terms that apply to them. kfk does not decide that
for you.

## No pool

Add nothing, and everything runs on the login Claude Code already has (`claude` → `/login`). kfk calls this the
slot `login`. It still tracks that login's usage windows and shows them in `kfk accounts`.

## Adding an account

```sh
kfk auth 0
```

1. kfk runs `claude setup-token`. A browser opens: sign in as the account you want in slot 0. Do not resize the
   terminal window while it runs.
2. kfk reads the token back, checks its shape and makes one small call with it to prove it works.
3. It asks for a label; type the account's email.
4. The token is stored in the macOS Keychain (service `kfk`, account `slot0`). It is never written to the kfk
   database, and it is never passed on a command line.

```
slot(s) 0 = you@example.com, verified ok
```

Slots are numbers from 0 to 99. Add as many as you have accounts:

```sh
kfk auth 1
kfk auth 2
```

Other ways in:

```sh
kfk auth 3 --paste                       # paste a token you already minted with `claude setup-token`
kfk auth 3 --label you@example.com       # skip the label question
kfk auth 4,5                             # the same account in two slots (rarely useful)
kfk auth 3 --from-log <file>             # retry from a kept sign-in transcript (kfk prints this when needed)
```

Never run `/login` inside a fleet window: it would replace the login of that one session and confuse the accounting.

## The pool

```sh
kfk accounts        # or: kfk ls
```

```
slot  account          minted      state  usage           resets     seen  seats  agents
0     a@example.com    2026-11-22  ok     5h 12%, wk 33%  Sun 21:00  2m    cos    1
1     b@example.com    2026-11-22  ok     wk 98%          Sat 23:00  31m   -      -
2     c@example.com    2026-11-22  ok     wk FULL         Wed 00:00  23m   -      -
3     d@example.com    2026-11-22  off    wk 4%           Mon 04:00  1h    -      -
```

| Column | Meaning |
| --- | --- |
| `state` | `ok`; `off` (you took it out of the choice); `NO TOKEN` (the Keychain entry is gone: `kfk auth <slot>`). |
| `usage` | Each live usage window: `5h` is the five-hour window, `wk` the weekly one; others are per-model. `FULL` means the server rejected a request. |
| `resets` | When the fullest window resets. |
| `seen` | How long ago kfk last learned anything about this account. |
| `seats`, `agents` | Which nodes and how many agents are on it now. |

## How usage is learned

kfk never polls. It reads what the sessions say while they work:

- every agent's output stream carries rate-limit events with the utilisation and reset time of each window;
- every node's status line carries the same for its account.

So an account that has not been used recently shows what was true when it last ran. `kfk check` makes one small call
per account to refresh:

```sh
kfk check          # every slot
kfk check 0,2
```

```
slot 0: ok
slot 2: REJECTED (mint again: kfk auth 2)
```

It exits 1 if any slot is not ok.

## How an account is chosen

For every new agent and every node start, kfk scores each account that is on and has a token:

```
score = worst live window utilisation (in %; 30 if nothing is known yet)
        + 3 for each agent running on it
        + 5 for each node sitting on it
```

The lowest score wins. Accounts at or above the limit are skipped: `agent_max_util` (97%) for agents,
`node_max_util` (95%) for nodes. Windows that only bind one model family (for example a weekly limit on one model)
count only for requests on that family.

The choice and the concurrency gate happen in one database transaction, so parallel runners never over-commit an
account.

## Limits in mid-flight

- **An agent hits a limit or an auth error.** The attempt is recorded as `limit` and does not count. The account is
  kept out of the choice (until the window the server named resets; 15 minutes when it named none; an hour after an
  auth rejection), and the attempt is repeated on another account.
- **Every account is full.** By default the task becomes `blocked` with the earliest reset time. Set
  `kfk config capacity_wait_s 14400` and the runner instead waits up to that many seconds for a window to reset,
  which is what you want for runs that last days.
- **A node's account passes 95%.** mizan moves the node to another account and restarts it with
  `claude --resume`, so its conversation continues. At most three such restarts per node per hour.

## Managing

```sh
kfk off 1                 # never choose slot 1 for anything new (a node already on it stays)
kfk on 1
kfk rm 1                  # remove from the pool and the Keychain; asks for "yes" (--yes to skip)
kfk swap o1               # move orchestrator o1 to the best other account now; its session is kept
kfk swap cos 4            # move cos to slot 4
kfk swap o1 --no-restart  # only record it; takes effect at the node's next start
kfk seat ls               # which node is on which account
kfk seat set o2 3         # the same as swap --no-restart
```

Removing an account does not revoke its token; that token stays valid until it expires. Nodes that were on the
removed slot are moved to another account.

## Running something as an account

```sh
kfk as 2 claude                 # an interactive Claude Code session on slot 2
kfk as cos claude -p "hello"    # on whatever account cos is on
kfk as 0 claude --version
```

The first argument is a slot number or a seat name. The command gets the account's token in its environment
(`CLAUDE_CODE_OAUTH_TOKEN`) and nothing else from kfk, so treat anything you run this way as holding that account.
