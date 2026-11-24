# Security

kfk starts AI sessions that read and change your code and run shell commands, without a person approving each
step. Read this page before you point it at anything you care about.

## What sessions are allowed to do

By default every node and every agent is started with `--dangerously-skip-permissions`. That means:

- they run any shell command, and read and write any file your user account can, **without asking**;
- they have your network access, your SSH keys, your cloud credentials and whatever else your shell has;
- a prompt injection in a file, a web page or a dependency that an agent reads can become a command it runs.

This default exists because an unattended agent that stops to ask a question nobody is there to answer cannot
finish. It is a real trade. To reduce it:

- **Narrow the flags.** `node_flags` and `agent_flags` are yours to set
  ([Configuration](configuration.md#flags-passed-to-claude-code)).
- **Run kfk as a user that has little.** A separate macOS user, a virtual machine or a container limits what "any
  file your user can" means.
- **Keep secrets out of reach.** Do not leave production credentials in the environment or the repository that
  agents work in.
- **Review before it leaves the machine.** Have streams deliver branches or pull requests, not deploys. Tasks say
  "do not push" unless the spec says otherwise, but that is an instruction to a model, not a control.
- **Write checks that cannot be gamed cheaply**, and review the diff, not just the check.

What kfk does on its side:

- Each agent works in its own git worktree on its own branch, so parallel agents do not overwrite each other and
  nothing lands on your main branch by itself. A worktree is not a sandbox: an agent can still reach outside it.
- Sessions get a clean environment: `ANTHROPIC_API_KEY`, `ANTHROPIC_AUTH_TOKEN` and every `CLAUDE_CODE_*` variable
  of the calling shell are dropped. Other variables in your shell are inherited.
- Checks run without the account token in their environment.

## The folder trust dialog

Claude Code asks whether you trust a folder the first time it runs there. kfk never answers that dialog, and never
pre-accepts it. It tells you a node is waiting (`kfk status`, the web view) and you accept it yourself with
`kfk attach`. The same goes for the one-time bypass-permissions warning and for sign-in prompts.

## Account tokens

- A token is minted with `claude setup-token` and stored in the **macOS Keychain** (service `kfk`, account
  `slot<N>`). It is not in the kfk database and not in any file kfk writes.
- It is written to the Keychain through standard input of the `security` tool and handed to sessions through the
  environment. It never appears in a process's argument list.
- During `kfk auth`, the sign-in is recorded to a file in the data home so the token can be read back. The file is
  made readable only by you and is deleted as soon as the token is stored. If storing fails, the file is kept and
  kfk prints its path so you can retry; delete it afterwards.
- `kfk rm <slot>` deletes the Keychain entry. It does **not** revoke the token: revoke it from the account's own
  settings if it may have leaked.
- Any process running as you can ask the Keychain for these entries, and any session on an account holds that
  account's token in its environment. A session that can run shell commands can therefore read its own token.
- `KFK_TOKEN_FILE` stores tokens in a clear-text JSON file. It exists for the test suite. Do not use it for real
  accounts.

Using several accounts is your responsibility: make sure it is allowed by the terms that apply to them.

## The web view

- No authentication, no TLS.
- Listens on `127.0.0.1` by default. Bind it elsewhere only behind something that authenticates.
- The pages are read-only. The one route that changes anything is the terminal of a node
  (`/ws/node/<seat>`): it types into that node's session, and nodes run with the permissions described above.
- A terminal is served only when all of this holds: `web_shell` is not `off`; the server listens on a loopback
  address; the request's `Host` names this machine (which defeats DNS rebinding); and its `Origin`, when a browser
  sends one, is the web view itself (which stops other sites' pages). Otherwise the answer is `403` and the reason.
- A terminal is a tmux client, so whoever can open one can do anything you can do in a shell on that machine, not
  only talk to Claude.
- That guard is about the network and the browser, not about local users: there is no password, so any process on
  the machine that can open `127.0.0.1` can open a terminal. On a shared machine, run
  `kfk config web_shell off`.
- It shows prompts, transcripts and check output, which can contain whatever your code and tools print, including
  secrets an agent happened to display.

## The log archive

- A private PostgreSQL cluster in the data home, reachable only through a Unix socket there; it opens no network
  port.
- It uses trust authentication on that socket: any process running as you can read it. The transcript files it
  copies have the same exposure.
- It keeps everything for ever, by design. A secret that appeared in a transcript stays in the archive until you
  delete the rows yourself.

## What leaves your machine

kfk itself makes no network requests. The Claude Code sessions it starts talk to Anthropic's service exactly as
they would if you ran them by hand, and whatever tools an agent runs can reach whatever your network allows.

## Reporting a vulnerability

Please do not open a public issue for a security problem. Report it privately to the maintainers through the
repository's security advisory page, with steps to reproduce.
