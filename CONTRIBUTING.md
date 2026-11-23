# Contributing to kfk

Thank you for helping. kfk is small on purpose; the bar for a change is that it makes the fleet deliver more
checked work per token, or makes kfk easier to run.

## Before you start

- For anything larger than a fix, open an issue first and say what problem it solves. It saves you from building
  something that does not fit.
- Read [docs/architecture.md](docs/architecture.md) and [docs/development.md](docs/development.md).

## Making a change

```sh
git clone <your fork> && cd kfk
cargo test                 # must pass before you change anything
# ... change, with tests ...
tests/coverage.sh          # must end with: core logic: 100% of lines
```

What a good change looks like:

- **It comes with its test**, written to state the behaviour. A bug fix starts with a test that fails.
- **Core logic stays at 100% line coverage.** The gate enforces it. If your code talks to a process, a socket or
  the Keychain, keep the decision in a pure function and the call in an edge file.
- **Files stay small** and do one thing. A new concern is a new file; a new subsystem is a new crate.
- **The docs move with the code.** A new command goes in `docs/commands.md`, a new key in
  `docs/configuration.md`, a new failure in `docs/troubleshooting.md`.
- **Messages help.** An error says what happened and which command fixes it.
- **Role prompts stay short.** Every added line costs tokens on every turn of every node. Say why it earns them.

Please do not:

- add an LLM call to mizan, or to anything that can be a rule;
- add a background service, a network listener or a dependency without discussing it first;
- make kfk answer a permission, trust or sign-in dialog on the user's behalf;
- put tokens in files, logs, argument lists or the database.

## Pull requests

- One topic per pull request.
- Describe what changed and how you verified it: paste the last lines of `tests/coverage.sh`.
- If the change affects how the fleet behaves with real Claude (role prompts, routing, rotation), say what you ran
  and what `kfk report` showed before and after.

## Reporting a bug

Include:

- `kfk status` and, for a task, `kfk task show <id>`;
- the relevant file from `<data home>/log/`;
- your macOS, Claude Code (`claude --version`), tmux and Rust versions;
- what you expected.

Remove account emails, tokens and anything private from what you paste. Transcripts can contain secrets your tools
printed.

Security problems: see [docs/security.md](docs/security.md#reporting-a-vulnerability); please do not open a public
issue.

## License

By contributing, you agree that your contribution is dual licensed under the MIT license and the Apache License,
Version 2.0, as described in the [README](README.md#license), without any additional terms or conditions.
