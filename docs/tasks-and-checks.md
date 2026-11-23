# Tasks and checks

Orchestrators write tasks for you, but you can write them yourself, and knowing what makes a good one helps you
judge what the fleet does. The quality of the check decides the quality of the delivery.

## Adding a task

```sh
kfk task add "<title>" \
  --spec  "<what and why: files, interfaces, constraints>" \
  --dod   "<the observable end state>" \
  --check '<a shell command; exit 0 = done>' \
  [--class mech|impl|hard|deep] [--repo <path>] [--base <branch>] \
  [--parent <stream>] [--owner <orchestrator>] [--dep <id>]... \
  [--model <m>] [--effort <e>] [--timeout <seconds>] [--max-attempts <n>]
```

It prints the new id, for example `T12`. Ids are accepted everywhere as `12` or `T12`.

`--spec` and `--dod` take text, `-` to read standard input, or `@file` to read a file:

```sh
kfk task add "rate limiter" --spec @docs/rate-limiter-brief.md --dod "..." --check 'cargo test -p api limiter'
git diff main | kfk task add "review fixes" --spec - --dod "..." --check '...'
```

A child inherits `owner` and `repo` from its parent unless it sets them.

## Writing the spec

The agent starts with no memory of anything. The spec is all it knows.

- **Name the files.** "Add `slugify(text) -> str` in `textkit/slugify.py`" beats "add a slugify helper".
- **Give the interface.** Function signatures, endpoint shapes, the error to raise.
- **Give examples.** `slugify("Hello, World!") == "hello-world"` settles more than a paragraph.
- **State the constraints.** Language version, allowed dependencies, files not to touch.
- **Say how to run the tests** if it is not obvious from the repository.

Keep one task to one independently verifiable change. If you write "and" between two unrelated things, make two
tasks.

## Writing the check

A check is a shell command run with `bash -c` in the task's worktree. Exit status 0 means done.

Good checks:

```sh
--check 'cargo test -p billing invoice_rounding'
--check 'npm test -- --run src/auth && npx tsc --noEmit'
--check 'test -f docs/api.md && grep -q "## Rate limits" docs/api.md'
--check './scripts/e2e.sh checkout'
```

Rules of thumb:

- **It fails now and passes after.** Run it before you add the task. A check that already passes proves nothing.
- **It tests the outcome, not the effort.** "A file exists" is weak; "the test that exercises it passes" is strong.
- **It is specific.** Run the tests for this change, plus whatever must not regress.
- **It is self-contained.** It gets no input and cannot ask anything. It has 15 minutes (`check_timeout_s`).
- **It does not depend on the agent's goodwill.** If the agent could pass it by deleting a test, add
  `git diff --stat main -- tests/ | grep -q .` or pin the test file's existence.

The check sees two extra environment variables: `KFK_TASK` (the task id) and `KFK_DIR` (the directory it runs in).

Checks are not for "looks right". If you cannot write a command for a goal, the goal is not ready: narrow it.

## Choosing the class

| Class | Use it for | Starts at |
| --- | --- | --- |
| `mech` | Renames, config, moving code, boilerplate | haiku / low |
| `impl` | A well-specified change. The default. | sonnet / medium |
| `hard` | Changes across modules, tricky debugging | opus / high |
| `deep` | Design-level work, an unknown root cause | fable / high |

Pick the cheapest class that can pass. A failed check escalates by itself, so starting low costs one cheap attempt
at worst, while starting high costs every time.

## Dependencies

Tasks that touch the same files should run in order:

```sh
kfk task add "schema: add invoices.currency" --parent 3 --check '...'           # T4
kfk task add "API: accept currency"          --parent 3 --dep 4 --check '...'   # T5
kfk task add "UI: currency picker"           --parent 3 --dep 5 --check '...'   # T6
kfk task add "docs: currency"                --parent 3 --check '...'           # T7, runs alongside T4
```

`kfk exec 3` starts every child whose dependencies are done, up to `max_agents` at a time. A child whose
dependency failed stays `todo` and is reported as waiting.

Each task has its own branch. A task that depends on another does **not** automatically start from the other's
branch: set `--base kfk/T4` on T5 if it needs T4's commits.

## Running

```sh
kfk run 12            # one task, in the foreground, until it settles
kfk run 12 --detach   # in the background; then: kfk wait 12
kfk exec 3            # every open child of stream 3, in dependency order, in parallel
kfk exec 3 --jobs 2   # at most two agents at once for this stream
kfk exec 3 --detach
```

`kfk exec` prints one summary when everything has settled:

```
id  status  tries  last model     weighted  result
T4  done    1      haiku/low      19k       branch kfk/T4, check passed on attempt 1 (haiku/low, slot 2)
T5  done    2      opus/high      61k       branch kfk/T5, check passed on attempt 2 (opus/high, slot 0)
T6  failed  3      fable/high     212k      check still failing after 3 attempts
T7  done    1      sonnet/medium  22k       branch kfk/T7, check passed on attempt 1 (sonnet/medium, slot 2)
stream T3: 3/4 done, 1 failed, 0 blocked | 314k weighted tokens
```

Exit status: 0 when every child that was not cancelled is done, 3 otherwise.

Runs are detached from your terminal and from the orchestrator that started them. They survive a closed window, a
restarted node and `kfk down --force`.

## When a task fails

```sh
kfk task show 6
```

shows the brief, every attempt, the tail of the last failing check and the agent's last words. Then one of:

```sh
kfk task set 6 spec=@better-spec.md          # fix the brief
kfk task set 6 class=hard                    # start higher
kfk task set 6 check='cargo test -p ui currency_picker -- --test-threads=1'
kfk task cancel 6 && kfk task add ...        # split it
```

and then run it again:

```sh
kfk task reopen 6 && kfk exec 3              # exec runs the children that are todo; done ones are left alone
kfk run 6                                    # or just this one
```

Either way the attempt count starts again from the first rung. The worktree is kept, so the next agent starts
from what the last one left.

## Finishing a stream

```sh
kfk task done 3 --result "PR #482"
```

kfk first runs stream 3's own check, in its repository. If the check fails, the stream stays open and the command
exits 3 with the tail of the output. `--force` only overrides the guard against open children; it does not skip the
check.

Other endings:

```sh
kfk task fail 3 --result "the upstream API does not expose currency"
kfk task block 3 --result "needs owner: USD only, or multi-currency?"   # shows under "Needs you"
kfk task cancel 3
kfk task reopen 3
kfk task note 3 "decided: store amounts in minor units"                # a decision, kept with the task
```

## Listing

```sh
kfk task                        # open tasks
kfk task ls --streams           # streams only
kfk task ls --parent 3          # one stream's children
kfk task ls --owner o1 --open
kfk task ls --status failed
kfk task ls --json | jq '.[] | select(.attempts > 1) | .title'
kfk task show 6 --json
```
