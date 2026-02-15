#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
E2E_DIR="$ROOT/crates/but-engineering-rewrite/e2e"
PROMPTS_DIR="$E2E_DIR/prompts"

# Real CLI (preferred). For local dev machines without Rust, set BIN=... to a prebuilt binary.
BIN="${BIN:-$ROOT/target/debug/but-engineering-rewrite}"
STUB_BIN="$ROOT/crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite"

SCENARIO="smoke"
PROVIDER="codex" # codex|claude
TIMEBOX_S=240
ALLOW_STUB=0
NO_AGENTS=0
KEEP_REPO=0

usage() {
  cat <<'EOF'
Usage: crates/but-engineering-rewrite/e2e/run.sh [options]

Options:
  --scenario <smoke|collision|discovery>
  --provider <codex|claude>
  --timebox-s <seconds>
  --allow-stub        Allow using the harness stub binary if the real binary isn't available.
  --no-agents         Skip spawning codex/claude (useful for offline smoke checks).
  --keep-repo         Keep the temp git repo (prints its path at end).

Env:
  BIN=...             Path to a real but-engineering-rewrite binary (preferred).

Notes:
  - This is an opt-in slow suite. It creates a temp git repo and writes outputs under e2e/out/.
  - The default scenario is a deterministic CLI smoke test plus an optional "agent spawned" smoke.
  - collision: minimal E2E-01 slice (two-agent collision -> read/ack -> release -> proceed). Provider=codex.
  - discovery: minimal E2E-02 slice (A posts discovery -> B brief/digest surfaces -> B executes suggested_action.cmd).
EOF
}

bold() { printf "\033[1m%s\033[0m\n" "$*"; }
fail() { printf "FAIL: %s\n" "$*" >&2; exit 1; }

now_ms() { python3 -c 'import time; print(int(time.time()*1000))'; }

mk_out_dir() {
  local out_root="$E2E_DIR/out"
  mkdir -p "$out_root"
  local ts
  ts="$(python3 -c 'import datetime; print(datetime.datetime.utcnow().strftime("%Y%m%dT%H%M%SZ"))')"
  OUT_DIR="$out_root/$ts.$$"
  mkdir -p "$OUT_DIR"
  TRACE_PATH="$OUT_DIR/trace.jsonl"
  : >"$TRACE_PATH"
}

json_escape() {
  local s="$1"
  s="${s//\\/\\\\}"
  s="${s//\"/\\\"}"
  s="${s//$'\n'/\\n}"
  s="${s//$'\r'/\\r}"
  s="${s//$'\t'/\\t}"
  printf "%s" "$s"
}

json_array() {
  local out="[" sep=""
  local s
  for s in "$@"; do
    out+="$sep\"$(json_escape "$s")\""
    sep=","
  done
  out+="]"
  printf "%s" "$out"
}

trace_emit() {
  local label="$1"
  local cwd="$2"
  local ec="$3"
  local cmd="$4"
  local args_json="$5"
  local stdout="$6"
  local stderr="$7"

  local ts_ms
  ts_ms="$(now_ms)"

  printf '{' >>"$TRACE_PATH"
  printf '"ts_ms":%s' "$ts_ms" >>"$TRACE_PATH"
  printf ',"label":"%s"' "$(json_escape "$label")" >>"$TRACE_PATH"
  printf ',"cwd":"%s"' "$(json_escape "$cwd")" >>"$TRACE_PATH"
  printf ',"cmd":"%s"' "$(json_escape "$cmd")" >>"$TRACE_PATH"
  printf ',"args":%s' "$args_json" >>"$TRACE_PATH"
  printf ',"exit_code":%s' "$ec" >>"$TRACE_PATH"
  printf ',"stdout":"%s"' "$(json_escape "$stdout")" >>"$TRACE_PATH"
  printf ',"stderr":"%s"' "$(json_escape "$stderr")" >>"$TRACE_PATH"
  printf '}\n' >>"$TRACE_PATH"
}

mk_repo() {
  local repo
  repo="$(mktemp -d "${TMPDIR:-/tmp}/but-erw-e2e.XXXXXX")"
  git -C "$repo" init -q
  git -C "$repo" config user.email "e2e@example.invalid"
  git -C "$repo" config user.name "E2E"
  mkdir -p "$repo/src"
  printf "hello\n" >"$repo/src/app.txt"
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "init"
  echo "$repo"
}

ensure_bin() {
  if [[ -x "$BIN" ]]; then
    return 0
  fi

  if command -v cargo >/dev/null 2>&1; then
    bold "Building but-engineering-rewrite..."
    (cd "$ROOT" && cargo build -p but-engineering-rewrite >/dev/null)
    [[ -x "$BIN" ]] && return 0
  fi

  if [[ "$ALLOW_STUB" -eq 1 ]] && [[ -x "$STUB_BIN" ]]; then
    BIN="$STUB_BIN"
    return 0
  fi

  if ! command -v cargo >/dev/null 2>&1; then
    fail "cargo not found and BIN missing at $BIN (set BIN=... or pass --allow-stub)"
  fi
  fail "binary not found at $BIN (set BIN=... or pass --allow-stub)"
}

run_with_timeout() {
  local label="$1"
  local cwd="$2"
  local timebox_s="$3"
  shift 3

  local cmd="$1"
  shift 1
  local -a args=("$@")
  local args_json; args_json="$(json_array "${args[@]}")"

  local stdout_file="$OUT_DIR/$label.stdout"
  local stderr_file="$OUT_DIR/$label.stderr"
  : >"$stdout_file"
  : >"$stderr_file"

  local ec=0
  set +e
  python3 - "$timebox_s" "$cwd" "$stdout_file" "$stderr_file" "$cmd" "${args[@]}" <<'PY'
import os, subprocess, sys

timebox_s = int(sys.argv[1])
cwd = sys.argv[2]
stdout_path = sys.argv[3]
stderr_path = sys.argv[4]
cmd = sys.argv[5:]

with open(stdout_path, "wb") as out, open(stderr_path, "wb") as err:
    try:
        p = subprocess.run(cmd, cwd=cwd, stdout=out, stderr=err, timeout=timebox_s, check=False)
        sys.exit(p.returncode)
    except subprocess.TimeoutExpired:
        err.write(f"TIMEOUT after {timebox_s}s\n".encode("utf-8"))
        sys.exit(124)
PY
  ec=$?
  set -e

  local stdout stderr
  stdout="$(cat "$stdout_file" 2>/dev/null || true)"
  stderr="$(cat "$stderr_file" 2>/dev/null || true)"

  trace_emit "$label" "$cwd" "$ec" "$cmd" "$args_json" "$stdout" "$stderr"
  return "$ec"
}

spawn_agent() {
  local label="$1"
  local repo="$2"
  local prompt_file="$3"
  local expect_token="${4:-SMOKE_AGENT_OK}"

  if [[ "$NO_AGENTS" -eq 1 ]]; then
    trace_emit "$label" "$repo" 0 "(skipped)" "[]" "" ""
    return 0
  fi

  # Convention: run_with_timeout writes $OUT_DIR/$label.{stdout,stderr}.
  local stdout_file="$OUT_DIR/$label.stdout"
  local stderr_file="$OUT_DIR/$label.stderr"
  local ec=0

  case "$PROVIDER" in
    codex)
      command -v codex >/dev/null 2>&1 || fail "codex not found in PATH"
      run_with_timeout "$label" "$repo" "$TIMEBOX_S" \
        codex exec --ephemeral --full-auto --json -C "$repo" - <"$prompt_file"
      ec=$?
      ;;
    claude)
      command -v claude >/dev/null 2>&1 || fail "claude not found in PATH"
      # `claude -p` doesn't execute tools; this is a transcript-only smoke to validate we can spawn
      # the process and capture output deterministically. We'll iterate later with interactive/tool mode.
      run_with_timeout "$label" "$repo" "$TIMEBOX_S" \
        claude -p --no-session-persistence --output-format=text --input-format=text \
        --system-prompt "You are a software agent participating in an E2E smoke test." \
        "$(cat "$prompt_file")"
      ec=$?
      ;;
    *)
      fail "unknown --provider: $PROVIDER (expected codex|claude)"
      ;;
  esac

  [[ "$ec" -eq 0 ]] || return "$ec"

  # Deterministic verdict: ensure the agent emitted our sentinel token.
  # - claude: prompt demands exactly the token (no extra text).
  # - codex: output may be JSON events; accept the token anywhere in stdout/stderr.
  python3 - "$PROVIDER" "$expect_token" "$stdout_file" "$stderr_file" <<'PY'
import sys

provider = sys.argv[1]
token = sys.argv[2]
stdout_path = sys.argv[3]
stderr_path = sys.argv[4]

with open(stdout_path, "r", encoding="utf-8", errors="replace") as f:
    out = f.read()
with open(stderr_path, "r", encoding="utf-8", errors="replace") as f:
    err = f.read()

if provider == "claude":
    s = out.strip()
    if s != token:
        raise SystemExit(f"expected claude to reply with exactly {token!r}, got: {s!r}")
else:
    if token not in out and token not in err:
        raise SystemExit(f"expected agent output to contain {token!r}")
PY
}

scenario_smoke() {
  local repo="$1"

  # Copy the CLI into the temp repo so agent prompts can use a stable relative path.
  cp "$BIN" "$repo/but-engineering-rewrite"
  chmod +x "$repo/but-engineering-rewrite"

  run_with_timeout "cli.claim.A" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id A claim --path src/app.txt --ttl 15m

  run_with_timeout "cli.check.B" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id B check --path src/app.txt

  # Deterministic verdict: check output is JSON and contains the expected decision/reason.
  python3 - "$OUT_DIR/cli.check.B.stdout" <<'PY'
import json, sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as f:
    v = json.loads(f.read())

if v.get("decision") != "warn":
    raise SystemExit(f"expected decision=warn, got {v.get('decision')!r}")
if v.get("reason_code") != "claimed_by_other":
    raise SystemExit(f"expected reason_code=claimed_by_other, got {v.get('reason_code')!r}")
if "action_plan" not in v or not isinstance(v["action_plan"], list) or not v["action_plan"]:
    raise SystemExit("expected non-empty action_plan list")
PY

spawn_agent "agent.smoke" "$repo" "$PROMPTS_DIR/smoke.${PROVIDER}.txt"
}

scenario_collision() {
  local repo="$1"

  if [[ "$NO_AGENTS" -ne 1 ]] && [[ "$PROVIDER" != "codex" ]]; then
    fail "scenario collision requires --provider codex (or pass --no-agents for a CLI-only smoke)"
  fi

  # Copy the CLI into the temp repo so agent prompts can use a stable relative path.
  cp "$BIN" "$repo/but-engineering-rewrite"
  chmod +x "$repo/but-engineering-rewrite"

  # Agent A claims and posts a relevant update to seed the read/ack loop.
  run_with_timeout "cli.status.A" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id A status "editing src/app.txt"
  run_with_timeout "cli.plan.A" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id A plan "touch src/app.txt; will release when done"
  run_with_timeout "cli.claim.A" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id A claim --path src/app.txt --ttl 15m
  run_with_timeout "cli.post.A" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id A post "Working on src/app.txt; will update/release soon."

  if [[ "$NO_AGENTS" -eq 1 ]]; then
    run_with_timeout "cli.check.B1" "$repo" "$TIMEBOX_S" \
      "$repo/but-engineering-rewrite" --agent-id B check --path src/app.txt
    run_with_timeout "cli.read.B1" "$repo" "$TIMEBOX_S" \
      "$repo/but-engineering-rewrite" --agent-id B read
    run_with_timeout "cli.post.B1" "$repo" "$TIMEBOX_S" \
      "$repo/but-engineering-rewrite" --agent-id B post "@A: ack: saw your update re src/app.txt"
    run_with_timeout "cli.check.B1b" "$repo" "$TIMEBOX_S" \
      "$repo/but-engineering-rewrite" --agent-id B check --path src/app.txt
  else
    spawn_agent "agent.collision.step1" "$repo" "$PROMPTS_DIR/collision.step1.${PROVIDER}.txt" "E2E_COLLISION_STEP1_OK"
  fi

  # Verify: B posted an ack addressed to A mentioning the path.
  run_with_timeout "cli.read.messages" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id Z read
  python3 - "$OUT_DIR/cli.read.messages.stdout" <<'PY'
import json, sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as f:
    v = json.loads(f.read())

msgs = v.get("messages", [])
def text(m):
    if isinstance(m, dict):
        if "text" in m and isinstance(m["text"], str):
            return m["text"]
        if "raw" in m and isinstance(m["raw"], str):
            return m["raw"]
    return ""

ok = False
for m in msgs:
    if not isinstance(m, dict):
        continue
    if m.get("agent_id") != "B":
        continue
    t = text(m)
    tl = t.lower()
    if tl.startswith("@a") and "ack" in tl and "src/app.txt" in t:
        ok = True
        break

if not ok:
    raise SystemExit("expected an ack-like message from agent B addressed to A mentioning src/app.txt")
PY

  # Agent A posts a closure/update and releases the claim, then B proceeds.
  run_with_timeout "cli.post.A2" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id A post "Resolved src/app.txt; released."
  run_with_timeout "cli.release.A" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id A release --path src/app.txt

  if [[ "$NO_AGENTS" -ne 1 ]]; then
    spawn_agent "agent.collision.step2" "$repo" "$PROMPTS_DIR/collision.step2.${PROVIDER}.txt" "E2E_COLLISION_STEP2_OK"
  fi

  run_with_timeout "cli.check.B2" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id B check --path src/app.txt
  python3 - "$OUT_DIR/cli.check.B2.stdout" <<'PY'
import json, sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as f:
    v = json.loads(f.read())

if v.get("decision") != "allow":
    raise SystemExit(f"expected decision=allow after release, got {v.get('decision')!r}")
if v.get("reason_code") != "no_conflict":
    raise SystemExit(f"expected reason_code=no_conflict, got {v.get('reason_code')!r}")
PY
}

scenario_discovery() {
  local repo="$1"

  if [[ "$NO_AGENTS" -ne 1 ]] && [[ "$PROVIDER" != "codex" ]]; then
    fail "scenario discovery requires --provider codex (or pass --no-agents for a CLI-only run)"
  fi

  # Copy the CLI into the temp repo so agent prompts can use a stable relative path.
  cp "$BIN" "$repo/but-engineering-rewrite"
  chmod +x "$repo/but-engineering-rewrite"

  # Post a valid structured discovery with a deterministic suggested action.
  local discovery_json
  discovery_json="$(python3 - <<'PY'
import json

payload = {
  "signal": "high",
  "title": "High-signal: src/app.txt coordination smoke",
  "evidence": [{"kind": "path", "path": "src/app.txt"}],
  "suggested_action": {
    "cmd": "./but-engineering-rewrite --agent-id B post \"@A: ack: executed discovery next_step for src/app.txt\""
  },
}
print(json.dumps(payload, separators=(",",":")))
PY
)"

  run_with_timeout "cli.post.discovery.A" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id A post --type discovery --json "$discovery_json"

  run_with_timeout "cli.brief.B" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id B brief
  python3 - "$OUT_DIR/cli.brief.B.stdout" <<'PY'
import json, sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as f:
    v = json.loads(f.read())

discoveries = v.get("discoveries", [])
if not discoveries:
    raise SystemExit("expected brief.discoveries to be non-empty")

ok = False
for d in discoveries:
    if not isinstance(d, dict):
        continue
    if d.get("agent_id") != "A":
        continue
    if d.get("title") == "High-signal: src/app.txt coordination smoke":
        ok = True
        break
if not ok:
    raise SystemExit("expected brief to surface A's discovery with the expected title")

steps = v.get("next_steps", [])
if not steps or not isinstance(steps, list):
    raise SystemExit("expected brief.next_steps to be a non-empty list")

cmd = steps[0].get("cmd") if isinstance(steps[0], dict) else None
expected = './but-engineering-rewrite --agent-id B post "@A: ack: executed discovery next_step for src/app.txt"'
if cmd != expected:
    raise SystemExit(f"expected next_steps[0].cmd to match exactly. got={cmd!r}")
PY

  run_with_timeout "cli.digest.B" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id B digest
  python3 - "$OUT_DIR/cli.digest.B.stdout" <<'PY'
import json, sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as f:
    v = json.loads(f.read())

discoveries = v.get("discoveries", [])
if not discoveries:
    raise SystemExit("expected digest.discoveries to be non-empty")

ok = False
for d in discoveries:
    if not isinstance(d, dict):
        continue
    if d.get("agent_id") == "A" and d.get("title") == "High-signal: src/app.txt coordination smoke":
        ok = True
        break
if not ok:
    raise SystemExit("expected digest to include A's discovery (title + agent_id)")
PY

  if [[ "$NO_AGENTS" -eq 1 ]]; then
    run_with_timeout "cli.exec.next_step.B" "$repo" "$TIMEBOX_S" \
      bash -lc './but-engineering-rewrite --agent-id B post "@A: ack: executed discovery next_step for src/app.txt"'
  else
    spawn_agent "agent.discovery.step1" "$repo" "$PROMPTS_DIR/discovery.step1.${PROVIDER}.txt" "E2E_DISCOVERY_STEP1_OK"
  fi

  run_with_timeout "cli.read.messages.discovery" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id Z read
  python3 - "$OUT_DIR/cli.read.messages.discovery.stdout" <<'PY'
import json, sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as f:
    v = json.loads(f.read())

msgs = v.get("messages", [])
ok = False
for m in msgs:
    if not isinstance(m, dict):
        continue
    if m.get("agent_id") != "B":
        continue
    txt = m.get("text") or m.get("raw") or ""
    if isinstance(txt, str) and "executed discovery next_step" in txt:
        ok = True
        break

if not ok:
    raise SystemExit("expected a message from agent B indicating it executed the discovery next_step")
PY
}

main() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --scenario) SCENARIO="${2:-}"; shift 2 ;;
      --provider) PROVIDER="${2:-}"; shift 2 ;;
      --timebox-s) TIMEBOX_S="${2:-}"; shift 2 ;;
      --allow-stub) ALLOW_STUB=1; shift 1 ;;
      --no-agents) NO_AGENTS=1; shift 1 ;;
      --keep-repo) KEEP_REPO=1; shift 1 ;;
      -h|--help) usage; exit 0 ;;
      *) fail "unknown arg: $1 (try --help)" ;;
    esac
  done

mk_out_dir
bold "E2E out dir: $OUT_DIR"
bold "E2E trace: $TRACE_PATH"

ensure_bin

local repo
repo="$(mk_repo)"
bold "E2E repo: $repo"

set +e
case "$SCENARIO" in
  smoke) scenario_smoke "$repo" ;;
  collision) scenario_collision "$repo" ;;
  discovery) scenario_discovery "$repo" ;;
  *)
    printf "Unknown scenario: %s\n" "$SCENARIO" >&2
    printf "Supported: smoke, collision, discovery\n" >&2
    exit 2
    ;;
esac
local ec=$?
set -e

if [[ "$KEEP_REPO" -ne 1 ]]; then
  rm -rf "$repo" || true
else
  bold "Keeping repo: $repo"
fi

if [[ "$ec" -eq 0 ]]; then
  bold "E2E: PASS"
else
  bold "E2E: FAIL (exit $ec)"
fi
exit "$ec"
}

main "$@"
