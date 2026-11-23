#!/bin/bash
# end to end, against the built binary: a fake claude, a private tmux server, a file vault and a throwaway home
set -u
K=${KFK_BIN:-$(cd "$(dirname "$0")/.." && pwd)/target/debug/kfk}
T=$(mktemp -d "${KFK_E2E_TMP:-/tmp}/kfk-e2e.XXXXXX"); mkdir -p "$T/repo" "$T/claude"
# PostgreSQL for the log archive checks: $KFK_PG_BIN, else whatever is on PATH; without it those checks are skipped
PGBIN=${KFK_PG_BIN:-$(dirname "$(command -v initdb 2>/dev/null)" 2>/dev/null)}; [ -x "$PGBIN/initdb" ] || PGBIN=
export KFK_HOME=$T/h KFK_TOKEN_FILE=$T/tok.json KFK_CLAUDE=$(cd "$(dirname "$0")" && pwd)/fake-claude KFK_TMUX_SOCKET=kfk-e2e-$$
fails=0
ok() { if eval "$2"; then echo "ok   $1"; else echo "FAIL $1"; fails=$((fails+1)); fi; }
cleanup() { $K web stop >/dev/null 2>&1; $K logs stop >/dev/null 2>&1; $K mizan stop >/dev/null 2>&1; tmux -L "$KFK_TMUX_SOCKET" kill-server 2>/dev/null; [ "${KEEP:-}" ] || rm -rf "$T"; }
trap cleanup EXIT
cd "$T/repo" && git init -q && git -c user.email=t@t -c user.name=t commit -q --allow-empty -m init

echo '{"0":"tok-a","1":"tok-b"}' > "$KFK_TOKEN_FILE"
sqlite3 "$KFK_HOME/kfk.db" 'select 1' >/dev/null 2>&1; $K config mizan_every 1 >/dev/null
[ "$PGBIN" ] && $K config pg_bin "$PGBIN" >/dev/null
PORT=$((20000 + $$ % 20000)); $K config web_addr 127.0.0.1:$PORT >/dev/null
sqlite3 "$KFK_HOME/kfk.db" "insert into accounts(slot,label,minted) values(0,'a@x','2026-11-22'),(1,'b@x','2026-11-22')"
ok "accounts lists both slots" '[ "$($K accounts | grep -c "@x")" = 2 ]'

$K up --dir "$T/repo" >/dev/null
ok "up starts cos and mizan" '$K status | grep -qE "^cos +up" && $K mizan | grep -q running'
O=$($K orch start --model sonnet --effort medium)
ok "orch start names o1" '[ "$O" = o1 ]'
sleep 1
ok "cos and o1 are on different accounts" '[ "$($K status --json | python3 -c "import json,sys; print(len({s[\"slot\"] for s in json.load(sys.stdin)[\"seats\"]}))")" = 2 ]'
ok "node got its token and role" 'tmux -L $KFK_TMUX_SOCKET capture-pane -p -t "=kfk:=o1" | grep -q "fake node: o1 slot=[01] token=set"'

$K task add "stream one" --owner o1 --spec s --dod d --check 'true' >/dev/null
$K task add "easy" --parent 1 --class mech --spec "write good" --dod "out.txt is good" --check 'grep -q good out.txt' >/dev/null
$K task add "FLAKY one" --parent 1 --spec FLAKY --dod "out.txt is good" --check 'grep -q good out.txt' --dep 2 >/dev/null
$K task add "no check" --parent 1 --spec x --dod y >/dev/null
ok "exec refuses a child without a check" '! $K exec 1 2>/dev/null'
$K task cancel 4 >/dev/null
ok "exec runs the stream to done" '$K exec 1 | grep -q "stream T1: 2/2 done"'
ok "flaky task escalated sonnet -> opus" '$K task show 3 | grep -q "try 2: PASS | opus/high"'
ok "agents ran on pool accounts, not the login" '[ "$($K task show 2 --json | python3 -c "import json,sys; print(json.load(sys.stdin)[\"runs\"][0][\"slot\"])")" != -1 ]'
ok "ids are accepted as T<n>" '$K task show T3 | grep -q "^T3 "'
ok "worktree branch exists" 'git -C "$T/repo" rev-parse -q --verify kfk/T2 >/dev/null'
ok "task done runs the check" '$K task done 1 --result "branch x" | grep -q "check passed"'
ok "report counts three runs" '$K report | grep -q "tasks passed: 2 (1 on the first attempt) of 3 agent runs"'

# rotation: o1's account reaches its limit -> mizan moves it to another account, session kept
S=$(sqlite3 "$KFK_HOME/kfk.db" "select slot from seats where seat='o1'"); SID=$(sqlite3 "$KFK_HOME/kfk.db" "select sid from seats where seat='o1'")
sqlite3 "$KFK_HOME/kfk.db" "update accounts set windows='{\"seven_day\":{\"util\":99.0,\"reset\":1999999999,\"status\":null}}' where slot=$S; insert into accounts(slot,label) values(2,'c@x')"
echo '{"0":"tok-a","1":"tok-b","2":"tok-c"}' > "$KFK_TOKEN_FILE"
$K task add "keep o1 busy" --owner o1 >/dev/null; sleep 4
ok "mizan rotated o1 to another account" '[ "$(sqlite3 "$KFK_HOME/kfk.db" "select slot from seats where seat=\"o1\"")" != "$S" ] && $K mizan | grep -q rotate'
ok "rotation kept the session id" 'tmux -L $KFK_TMUX_SOCKET capture-pane -p -J -t "=kfk:=o1" | tr -d "\n" | grep -q -- "--resume $SID"'

# a dead node is restarted; an idle orchestrator is stopped
kill "$(sqlite3 "$KFK_HOME/kfk.db" "select pid from seats where seat='cos'")"; sleep 24
ok "mizan restarted cos" '$K status | grep -qE "^cos +up" && $K mizan | grep -q "node-start cos"'
$K task cancel 5 >/dev/null; $K config orch_idle_s 2 >/dev/null; sleep 6
ok "idle orchestrator stopped" '$K status | grep -qE "^o1 +stopped"'

if [ "$PGBIN" ]; then
# the log archive
mkdir -p "$T/claude/p" && printf '{"type":"user","timestamp":"2026-11-22T09:00:00.000Z"}\nnot json\n{"partial":' > "$T/claude/p/sess-1.jsonl"
$K config claude_projects "$T/claude" >/dev/null
ok "logs init" '$K logs init | grep -q "log archive ready"'
ok "logs sync archives sessions and agent streams" '$K logs sync | grep -qE "archived 1[0-9] new lines" && $K logs status | grep -q "oldest line 2026-11-22"'
printf '"x"}\n' >> "$T/claude/p/sess-1.jsonl"
ok "logs sync is incremental" '$K logs sync | grep -q "archived 1 new lines .* from 1 of"'
ok "an archived file reads back after it is gone from disk" 'rm "$T/claude/p/sess-1.jsonl" && $K logs cat "$T/claude/p/sess-1.jsonl" | grep -q "\"partial\": *\"x\"" && ! $K logs cat /no/such/file 2>/dev/null'
else echo "skip the log archive checks: no PostgreSQL (set KFK_PG_BIN)"; fi
$K mizan once >/dev/null; $K status >/dev/null; $K status --json >/dev/null; $K report >/dev/null; $K seat ls >/dev/null; $K task ls >/dev/null; $K config >/dev/null
ok "web starts and serves the overview" '$K web start | grep -qE "web (started|already running)" && sleep 1 && curl -sf http://127.0.0.1:$PORT/ | grep -q "stream one" && curl -sf http://127.0.0.1:$PORT/node/cos | grep -q "fake node: cos"'
ok "web serves a run with its transcript" 'curl -sf http://127.0.0.1:$PORT/run/1 | grep -q "did it"'
ok "web stops" '$K web stop | grep -q "web stopped" && sleep 1 && ! curl -sf -m 2 http://127.0.0.1:$PORT/healthz >/dev/null'
ok "down" '$K down | grep -q "^down" && ! tmux -L $KFK_TMUX_SOCKET has-session -t "=kfk" 2>/dev/null'
LAST=$(sqlite3 "$KFK_HOME/kfk.db" "select max(id) from events")
$K up >/dev/null; sleep 3
ok "up after down resumes the conversation of cos" 'tmux -L $KFK_TMUX_SOCKET capture-pane -p -J -t "=kfk:=cos" | tr -d "\n" | grep -q -- "--resume"'
ok "and starts it once, not twice" '[ "$(sqlite3 "$KFK_HOME/kfk.db" "select count(*) from events where kind=\"node-start\" and id > $LAST")" = 1 ]'
$K down >/dev/null; $K up --fresh >/dev/null; sleep 2
ok "up --fresh gives cos a new conversation" '! tmux -L $KFK_TMUX_SOCKET capture-pane -p -J -t "=kfk:=cos" | tr -d "\n" | grep -q -- "--resume"'
$K down >/dev/null
echo; [ $fails = 0 ] && echo "ALL PASSED" || echo "$fails FAILED"; exit $fails
