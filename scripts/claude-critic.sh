#!/usr/bin/env bash
set -euo pipefail

PROMPT_FILE=""
OUT="/tmp/claude-critic.out"
TIMEBOX_SECONDS="120"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prompt-file) PROMPT_FILE="$2"; shift 2;;
    --out) OUT="$2"; shift 2;;
    --timebox-seconds) TIMEBOX_SECONDS="$2"; shift 2;;
    *) shift;;
  esac
done

mkdir -p "$(dirname "$OUT")" 2>/dev/null || true

if [[ -z "$PROMPT_FILE" ]] || [[ ! -f "$PROMPT_FILE" ]]; then
  printf "claude-critic: missing --prompt-file (got: %s)\n" "$PROMPT_FILE" >"$OUT" 2>/dev/null || true
  exit 0
fi

if [[ ! -x /opt/homebrew/bin/claude ]]; then
  echo "claude-critic: claude binary not found at /opt/homebrew/bin/claude" >"$OUT" 2>/dev/null || true
  exit 0
fi

export PATH=/opt/homebrew/bin:$PATH

# Never fail caller
set +e

PROMPT_FILE="$PROMPT_FILE" TIMEBOX_SECONDS="$TIMEBOX_SECONDS" python3 - <<PY >"$OUT" 2>&1
import os, subprocess, sys

prompt_path = os.environ.get("PROMPT_FILE")
timebox = int(os.environ.get("TIMEBOX_SECONDS", "120"))

cmd = ["/opt/homebrew/bin/claude"]

try:
    p = subprocess.Popen(
        cmd,
        stdin=open(prompt_path, "rb"),
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
except Exception as e:
    print(f"claude-critic: failed to start claude: {e}")
    sys.exit(0)

try:
    out, _ = p.communicate(timeout=timebox)
except subprocess.TimeoutExpired:
    try:
        p.terminate()
    except Exception:
        pass
    try:
        out, _ = p.communicate(timeout=1)
    except Exception:
        out = b""
    try:
        p.kill()
    except Exception:
        pass
    print(f"claude-critic: timeboxed after {timebox}s (non-fatal)")
    sys.exit(0)

if not out or not out.strip():
    print(f"claude-critic: no output (exit={p.returncode})")
    sys.exit(0)

sys.stdout.buffer.write(out)
sys.exit(0)
PY

exit 0
