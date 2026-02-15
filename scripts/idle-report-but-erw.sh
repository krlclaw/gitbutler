#!/usr/bin/env bash
set -euo pipefail
REPO_DIR="${REPO_DIR:-$HOME/src/gitbutler}"
cd "$REPO_DIR"
SAMPLES_FILE=.metrics/idle-samples.jsonl
if [[ ! -f "$SAMPLES_FILE" ]]; then
  echo "No idle samples yet (missing $SAMPLES_FILE)."
  exit 0
fi
python3 - <<"PY"
import json, datetime
from pathlib import Path
p = Path(".metrics/idle-samples.jsonl")
cut = datetime.datetime.now(datetime.timezone.utc) - datetime.timedelta(minutes=60)
idle=busy=total=0
for line in p.read_text().splitlines():
    if not line.strip():
        continue
    try:
        o=json.loads(line)
        ts=datetime.datetime.fromisoformat(o["ts"].replace("Z","+00:00"))
        if ts < cut:
            continue
        total += 1
        if o.get("idle") is True:
            idle += 1
        else:
            busy += 1
    except Exception:
        continue
idle_pct = (idle/total*100.0) if total else 0.0
print(f"Last 60m samples={total} idle_minutes={idle} busy_minutes={busy} idle_pct={idle_pct:.0f}%")
PY

echo

echo "Last 5 commits:"
git log -5 --oneline || true

echo

echo "Next backlog (top):"
if [[ -f crates/but-engineering-rewrite/docs/08-backlog.md ]]; then
  sed -n 1,120p crates/but-engineering-rewrite/docs/08-backlog.md
else
  echo "(missing crates/but-engineering-rewrite/docs/08-backlog.md)"
fi
