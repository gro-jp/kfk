# Changelog

Versions are `v<0-9>.<0-99>.<0-99>`. Newest first.

## v0.0.1 (2026-11-24)

The first release. macOS only; built from source with `./install.sh`.

- **The fleet.** `kfk up` starts cos, a chief of staff, as an interactive Claude Code session in tmux. cos starts
  orchestrators on demand; orchestrators split a goal into small tasks and run short-lived agents on them.
- **Tasks with checks.** Every task has a shell check. An agent's work counts only when the check exits 0; a
  failed attempt is retried one rung up the model ladder. Each task gets its own git worktree and branch.
- **A pool of accounts.** `kfk auth` adds Claude subscription logins; tokens are kept in the Keychain. Each agent
  runs on the account with most headroom, and a run that hits a limit moves to another without losing an attempt.
- **mizan.** A daemon with no LLM in it: it watches nodes, restarts them, moves them between accounts, and
  records what they spend.
- **One local store.** Tasks, runs, sessions and events are in one SQLite file. Nothing is ever deleted.
- **Reports.** `kfk report` shows delivered work, tokens and the share spent on coordination.
- **Log archive.** `kfk logs` keeps every transcript in a private PostgreSQL cluster, synced incrementally.
- **Web view.** `kfk web start` serves an overview, nodes, tasks, accounts and a paginated history on
  `127.0.0.1:7878`, with a live two-way terminal for every node (this machine only; `web_shell` turns it off).
- **`kfk --version`.**

Known gaps are listed in the [README](README.md#status).
