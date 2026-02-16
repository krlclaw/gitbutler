#!/usr/bin/env bash
set -euo pipefail

CODEX_BIN="${BUT_EVAL_CODEX_BIN:-${BUT_EVAL_RUNNER_BIN:-codex}}"
PROMPT="${BUT_EVAL_PROMPT:-}"
MODEL="${BUT_EVAL_MODEL:-}"
APPEND_SYSTEM_PROMPT="${BUT_EVAL_APPEND_SYSTEM_PROMPT:-}"
MIN_CODEX_VERSION="${BUT_EVAL_MIN_CODEX_VERSION:-${BUT_EVAL_MIN_RUNNER_VERSION:-0.99.0}}"
RUNNER_TIMEOUT_MS="${BUT_EVAL_RUNNER_TIMEOUT_MS:-0}"

extract_semver() {
  local raw="$1"
  local semver
  semver="$(printf '%s\n' "$raw" | grep -Eo '[0-9]+\.[0-9]+\.[0-9]+' | head -n 1 || true)"
  printf '%s' "$semver"
}

semver_gte() {
  local left="$1"
  local right="$2"
  local l1 l2 l3 r1 r2 r3
  IFS=. read -r l1 l2 l3 <<<"$left"
  IFS=. read -r r1 r2 r3 <<<"$right"

  l1="${l1:-0}"
  l2="${l2:-0}"
  l3="${l3:-0}"
  r1="${r1:-0}"
  r2="${r2:-0}"
  r3="${r3:-0}"

  if ((l1 != r1)); then
    ((l1 > r1))
    return
  fi
  if ((l2 != r2)); then
    ((l2 > r2))
    return
  fi
  ((l3 >= r3))
}

if [[ -z "$PROMPT" ]]; then
  echo "BUT_EVAL_PROMPT is required" >&2
  exit 2
fi

if ! command -v "$CODEX_BIN" >/dev/null 2>&1; then
  echo "Codex CLI not found: $CODEX_BIN" >&2
  exit 2
fi

CODEX_PATH="$(command -v "$CODEX_BIN")"
CODEX_VERSION_RAW="$($CODEX_BIN --version 2>&1 || true)"
CODEX_VERSION="$(extract_semver "$CODEX_VERSION_RAW")"
if [[ -z "$CODEX_VERSION" ]]; then
  echo "Could not parse Codex CLI version from '$CODEX_VERSION_RAW' ($CODEX_PATH)." >&2
  exit 2
fi

if ! semver_gte "$CODEX_VERSION" "$MIN_CODEX_VERSION"; then
  echo "Codex CLI version $CODEX_VERSION at $CODEX_PATH is too old; need >= $MIN_CODEX_VERSION." >&2
  exit 2
fi

full_prompt="$PROMPT"
if [[ -n "$APPEND_SYSTEM_PROMPT" ]]; then
  full_prompt="$(printf 'Follow these policy requirements exactly:\n%s\n\nTask:\n%s\n' "$APPEND_SYSTEM_PROMPT" "$PROMPT")"
fi

args=(
  exec
  --json
  -c
  'approval_policy="never"'
  --sandbox
  danger-full-access
  --skip-git-repo-check
)

if [[ -n "$MODEL" ]]; then
  args+=(--model "$MODEL")
fi

args+=("$full_prompt")

if [[ "$RUNNER_TIMEOUT_MS" =~ ^[0-9]+$ ]] && [[ "$RUNNER_TIMEOUT_MS" -gt 0 ]]; then
  python3 - "$RUNNER_TIMEOUT_MS" "$CODEX_BIN" "${args[@]}" <<'PYRUN'
import os
import signal
import subprocess
import sys

if len(sys.argv) < 3:
    print('runner wrapper requires timeout and command', file=sys.stderr)
    sys.exit(2)

timeout_ms = int(sys.argv[1])
cmd = sys.argv[2:]
proc = subprocess.Popen(cmd, preexec_fn=os.setsid)

try:
    sys.exit(proc.wait(timeout=timeout_ms / 1000.0))
except subprocess.TimeoutExpired:
    try:
        os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
    except ProcessLookupError:
        pass
    print(f"Codex runner timed out after {timeout_ms}ms", file=sys.stderr)
    sys.exit(124)
PYRUN
else
  "$CODEX_BIN" "${args[@]}"
fi
