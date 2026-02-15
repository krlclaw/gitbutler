#!/usr/bin/env bash
set -euo pipefail
REPO_DIR="${REPO_DIR:-$HOME/src/gitbutler}"
cd "$REPO_DIR"
mkdir -p .metrics
TS=$(date -u +'%Y-%m-%dT%H:%M:%SZ')
PROCS=$(ps aux | grep -E 'codex|claude' | grep -v grep | grep -F "$REPO_DIR" || true)
if [[ -n "$PROCS" ]]; then
  IDLE=false
else
  IDLE=true
fi
printf '{"ts":"%s","idle":%s}\n' "$TS" "$IDLE" >> .metrics/idle-samples.jsonl
