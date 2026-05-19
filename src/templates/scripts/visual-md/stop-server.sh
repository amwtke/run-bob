#!/usr/bin/env bash
# Usage: stop-server.sh <session-dir>
set -euo pipefail
SESSION_DIR="${1:?session dir required}"
if [ -f "$SESSION_DIR/state/server.pid" ]; then
  PID=$(cat "$SESSION_DIR/state/server.pid")
  if kill -0 "$PID" 2>/dev/null; then
    kill "$PID" 2>/dev/null || true
    sleep 0.2
    kill -9 "$PID" 2>/dev/null || true
  fi
  rm -f "$SESSION_DIR/state/server.pid" "$SESSION_DIR/state/server-info"
  echo "stopped"
else
  echo "no server.pid found"
fi
