# Log archive

Claude Code deletes its session transcripts after a while (30 days by default). kfk can copy every transcript line
into a PostgreSQL database that it runs for itself, and never deletes anything from it.

The archive is optional. Without it, kfk works, and transcripts are read from disk for as long as the files exist.

## What is archived

| Source | Files |
| --- | --- |
| Every Claude Code session on this machine, kfk's or not | `~/.claude/projects/**/*.jsonl` |
| Every agent attempt | `<data home>/runs/<run>/stream.jsonl` |

Each line is stored once, keyed by file and byte offset, as `jsonb` when it is JSON and as text when it is not.
Archiving is incremental: only the bytes a file gained since last time are read.

## Set it up

You need PostgreSQL's command-line tools (`initdb`, `pg_ctl`, `postgres`). Any recent version works; kfk is tested
with 18.

```sh
brew install postgresql@17                # or build from source, below
kfk config pg_bin "$(brew --prefix postgresql@17)/bin"
kfk logs init
```

```
log archive ready at /Users/you/.kfk/pg (socket /Users/you/.kfk, port 54329): cluster created, server started, database created
```

If the tools are already on your `PATH`, leave `pg_bin` empty.

`kfk logs init` creates a private cluster at `<data home>/pg`. It listens on a Unix socket inside the data home and
on **no network address**. It does not touch any PostgreSQL you already run.

From then on mizan archives new lines every round. By hand:

```sh
kfk logs sync       # archive what is new, starting the server if needed
kfk logs            # status
kfk logs stop       # stop the server (kfk logs sync starts it again)
```

```
log archive: 73 files, 26377 lines, 113 MB | oldest line 2026-10-26 | last sync 2026-11-23 02:07
```

### Building PostgreSQL from source

```sh
curl -LO https://ftp.postgresql.org/pub/source/v18.7/postgresql-18.7.tar.bz2
tar xf postgresql-18.7.tar.bz2 && cd postgresql-18.7
./configure --prefix="$HOME/pg/18" --without-icu
make -j8 && make install
kfk config pg_bin "$HOME/pg/18/bin"
```

## Reading it back

A file that is gone from disk can be printed again:

```sh
kfk logs cat ~/.claude/projects/-Users-you-code-my-repo/92af8a07-….jsonl | jq -r 'select(.type=="user") | .message.content'
```

The web view does the same automatically for run and session pages.

## Querying

It is an ordinary database:

```sh
psql -h ~/.kfk -p 54329 -U kfk kfk
```

```sql
-- the tables
-- log_files(id, path, session, source, bytes_done, first_seen, last_sync)   source: 'session' | 'agent'
-- log_lines(file_id, off, ts, type, data jsonb, raw text)

-- output tokens per day, all sessions
select date_trunc('day', ts) as day, sum((data #>> '{message,usage,output_tokens}')::bigint) as out_tokens
from log_lines where type = 'assistant' group by 1 order by 1;

-- every Bash command an agent or session ever ran that mentions "rm -rf"
select f.path, l.ts, b ->> 'input'
from log_lines l join log_files f on f.id = l.file_id,
     jsonb_array_elements(l.data #> '{message,content}') b
where l.type = 'assistant' and b ->> 'name' = 'Bash' and b #>> '{input,command}' like '%rm -rf%';
```

A transcript repeats an assistant message once per content block, with the same usage on each line; count a message
id once when you add up tokens.

## Keeping the originals longer

The archive makes Claude Code's own clean-up harmless, but you can also turn that clean-up down. In
`~/.claude/settings.json`:

```json
{ "cleanupPeriodDays": 36500 }
```

## Notes

- **Size.** Expect roughly 4 MB per thousand lines. Put the data home on a disk with room.
- **Socket path length.** Unix socket paths are limited to about 100 characters. Keep the data home path short.
- **Backups.** `pg_dump -h ~/.kfk -p 54329 -U kfk kfk | gzip > kfk-logs.sql.gz`. The SQLite store is one file,
  `<data home>/kfk.db`; copy it with `sqlite3 ~/.kfk/kfk.db ".backup kfk-backup.db"`.
- **Access.** The cluster uses trust authentication on its socket: any process running as you can read it. That is
  the same exposure as the transcript files themselves.
- **Port.** `pg_port` (54329) only names the socket file; change it if it collides.
