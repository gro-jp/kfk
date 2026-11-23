#!/bin/bash
# Build kfk and install it, asking where it should keep its data.
#
#   ./install.sh                         ask, with defaults
#   ./install.sh --yes                   take every default, ask nothing
#   ./install.sh --prefix ~/.local --home /data/kfk --pg-bin /opt/pg/bin
#
# --prefix DIR   the binary goes to DIR/bin/kfk            (default: ~/.local)
# --home DIR     tasks, runs, logs and the databases       (default: the home already recorded, else ~/.kfk)
# --pg-bin DIR   PostgreSQL's bin directory, for the log   (default: found on PATH; empty = no archive for now)
#                archive
set -euo pipefail
cd "$(dirname "$0")"
export PATH="$HOME/.cargo/bin:$PATH"

pointer="${XDG_CONFIG_HOME:-$HOME/.config}/kfk/home"
prefix="$HOME/.local"; home=""; pgbin="unset"; yes=""
while [ $# -gt 0 ]; do
  case "$1" in
    --prefix) prefix=$2; shift 2 ;;
    --home) home=$2; shift 2 ;;
    --pg-bin) pgbin=$2; shift 2 ;;
    --yes|-y) yes=1; shift ;;
    -h|--help) sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "install.sh: unknown option $1 (see --help)" >&2; exit 2 ;;
  esac
done

need() { command -v "$1" >/dev/null 2>&1 || { echo "install.sh: $1 is required: $2" >&2; exit 1; }; }
warn() { command -v "$1" >/dev/null 2>&1 || echo "note: $1 is not on PATH: $2"; }
need cargo "install Rust from https://rustup.rs"
need git "kfk gives every task its own git worktree"
need tmux "cos and the orchestrators live in a tmux session"
warn claude "kfk drives the Claude Code CLI; install it and sign in before 'kfk up'"

ask() { # ask <question> <default>  -> answer on stdout
  if [ -n "$yes" ] || [ ! -t 0 ]; then echo "$2"; return; fi
  local a; read -r -p "$1 [$2]: " a; echo "${a:-$2}"
}
[ -n "$home" ] || home=$(ask "Data home (tasks, runs, logs, databases)" "$( [ -s "$pointer" ] && cat "$pointer" || echo "$HOME/.kfk")")
if [ "$pgbin" = unset ]; then
  found=$(dirname "$(command -v initdb 2>/dev/null)" 2>/dev/null || true); [ -x "$found/initdb" ] || found=""
  pgbin=$(ask "PostgreSQL bin directory for the log archive (empty: set it up later)" "$found")
fi
case "$home" in /*) ;; *) echo "install.sh: the data home must be an absolute path (got $home)" >&2; exit 2 ;; esac

echo "building kfk (release)..."
cargo build --release --bin kfk
mkdir -p "$prefix/bin" "$home" "$(dirname "$pointer")"
cp target/release/kfk "$prefix/bin/kfk.new" && mv "$prefix/bin/kfk.new" "$prefix/bin/kfk"   # never write over a running binary
printf '%s\n' "$home" > "$pointer"
[ -z "$pgbin" ] || "$prefix/bin/kfk" config pg_bin "$pgbin" >/dev/null

echo
echo "installed: $prefix/bin/kfk"
echo "data home: $home   (recorded in $pointer; \$KFK_HOME overrides it)"
[ -n "$pgbin" ] && echo "PostgreSQL: $pgbin   (kfk logs init creates the archive)" || echo "PostgreSQL: not set; the log archive is optional (kfk config pg_bin <dir>, then kfk logs init)"
case ":$PATH:" in *":$prefix/bin:"*) ;; *) echo "note: $prefix/bin is not on your PATH" ;; esac
echo
echo "next:  kfk auth 0            # optional: add accounts to the pool (repeat for 1, 2, ...)"
echo "       kfk up --dir <repo>   # start cos, mizan and the web view"
echo "       kfk attach            # talk to cos"
