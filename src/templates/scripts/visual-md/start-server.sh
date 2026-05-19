#!/usr/bin/env bash
# Usage: start-server.sh <session-dir>
# Starts visual-md server in background; writes pid + server-info to <session-dir>/state/.
set -euo pipefail
SESSION_DIR="${1:?session dir required}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$SESSION_DIR/state"

# Stop any prior server in this session
if [ -f "$SESSION_DIR/state/server.pid" ]; then
  OLD_PID=$(cat "$SESSION_DIR/state/server.pid")
  if kill -0 "$OLD_PID" 2>/dev/null; then
    kill "$OLD_PID" 2>/dev/null || true
    sleep 0.3
  fi
  rm -f "$SESSION_DIR/state/server.pid" "$SESSION_DIR/state/server-info"
fi

# Do NOT pin VISUAL_MD_OWNER_PID to $PPID — when invoked from a tool's per-call
# subshell (e.g. Claude Code's Bash tool), $PPID dies immediately and the server
# self-terminates within 60s. Rely on the 30-min idle timeout for cleanup, plus
# explicit stop-server.sh. To opt in to PID watching, set VISUAL_MD_OWNER_PID
# in the env before invoking this script.
VISUAL_MD_DIR="$SESSION_DIR" \
nohup node "$SCRIPT_DIR/server.cjs" > "$SESSION_DIR/state/server.log" 2>&1 &
SERVER_PID=$!
echo "$SERVER_PID" > "$SESSION_DIR/state/server.pid"
disown "$SERVER_PID" 2>/dev/null || true

# Wait up to 3s for server-info to appear
for i in 1 2 3 4 5 6; do
  if [ -f "$SESSION_DIR/state/server-info" ]; then
    cat "$SESSION_DIR/state/server-info"
    exit 0
  fi
  sleep 0.5
done
echo "ERROR: server failed to write server-info within 3s" >&2
cat "$SESSION_DIR/state/server.log" >&2
exit 1
