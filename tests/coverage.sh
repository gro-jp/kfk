#!/bin/bash
# the coverage gate: core logic must be at 100% of lines; process edges (tmux, PostgreSQL, the CLI glue, the
# daemon) are exercised by the end-to-end test and reported, not gated
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
out=target/kfk-cov.json
if ! cargo llvm-cov --workspace --json --output-path "$out" >target/kfk-cov.log 2>&1; then
  grep -nE "FAIL|panicked|^error" -A8 target/kfk-cov.log | head -60; echo "tests failed under coverage: target/kfk-cov.log"; exit 1
fi
python3 - "$out" <<'PY'
import json, re, sys
EDGE = re.compile(r"kfk-cli/|kfk-web/src/(live|serve)\.rs|kfk-engine/src/detach\.rs|kfk-mizan/src/(tmux|node|node_main|round|slot|daemon)\.rs|kfk-logs/src/(pg|sync|status|read)\.rs")
bad, core, edge = [], [0, 0], [0, 0]
for f in json.load(open(sys.argv[1]))["data"][0]["files"]:
    name = f["filename"].split("/crates/")[-1]
    l = f["summary"]["lines"]
    tot = edge if EDGE.search(name) else core
    tot[0] += l["covered"]; tot[1] += l["count"]
    if not EDGE.search(name) and l["covered"] != l["count"]:
        bad.append(f"{name}: {l['covered']}/{l['count']} lines")
    if EDGE.search(name):
        print(f"edge  {name}: {100*l['covered']//max(l['count'],1)}%")
print(f"core: {core[0]}/{core[1]} lines | edges: {edge[0]}/{edge[1]} lines ({100*edge[0]//max(edge[1],1)}%)")
if bad:
    print("NOT 100%:\n  " + "\n  ".join(bad)); sys.exit(1)
print("core logic: 100% of lines")
PY
