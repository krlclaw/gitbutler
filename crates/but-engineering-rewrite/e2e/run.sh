#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
E2E_DIR="$ROOT/crates/but-engineering-rewrite/e2e"
PROMPTS_DIR="$E2E_DIR/prompts"

# Ensure non-interactive shells can find tool CLIs (Homebrew + user-local).
export PATH="/opt/homebrew/bin:/usr/local/bin:$HOME/.local/bin:$PATH"

# Real CLI (preferred). For local dev machines without Rust, set BIN=... to a prebuilt binary.
BIN="${BIN:-$ROOT/target/debug/but-engineering-rewrite}"
STUB_BIN="$ROOT/crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite"

SCENARIO="smoke"
PROVIDER="codex" # codex|claude|both (both only supported for smoke)
TIMEBOX_S=240
ALLOW_STUB=0
NO_AGENTS=0
KEEP_REPO=0
OUT_DIR_OVERRIDE=""

usage() {
  cat <<'EOF'
Usage: crates/but-engineering-rewrite/e2e/run.sh [options]

Options:
  --scenario <smoke|collision|discovery|triangle>
  --provider <codex|claude|both>  (both only supported for smoke)
  --timebox-s <seconds>
  --out-dir <path>     Write outputs to this directory (default: e2e/out/<ts>.<pid>)
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
  - triangle: minimal E2E-03 slice (3-agent triangle conflict + dependency hints noise control). Provider=codex.
EOF
}

bold() { printf "\033[1m%s\033[0m\n" "$*"; }
fail() { printf "FAIL: %s\n" "$*" >&2; exit 1; }

now_ms() { python3 -c 'import time; print(int(time.time()*1000))'; }

mk_out_dir() {
  local out_root="$E2E_DIR/out"
  mkdir -p "$out_root"
  if [[ -n "$OUT_DIR_OVERRIDE" ]]; then
    if [[ "$OUT_DIR_OVERRIDE" == /* ]]; then
      OUT_DIR="$OUT_DIR_OVERRIDE"
    else
      OUT_DIR="$ROOT/$OUT_DIR_OVERRIDE"
    fi
  else
    local ts
    ts="$(python3 -c 'import datetime; print(datetime.datetime.utcnow().strftime("%Y%m%dT%H%M%SZ"))')"
    OUT_DIR="$out_root/$ts.$$"
  fi
  mkdir -p "$OUT_DIR"
  TRACE_PATH="$OUT_DIR/trace.jsonl"
  : >"$TRACE_PATH"

  META_PATH="$OUT_DIR/meta.json"
  python3 - "$META_PATH" "$ROOT" "$E2E_DIR" "$SCENARIO" "$PROVIDER" "$TIMEBOX_S" "$ALLOW_STUB" "$NO_AGENTS" "$KEEP_REPO" "$BIN" "$STUB_BIN" <<'PY'
import json, os, shutil, subprocess, sys

out_path = sys.argv[1]
root = sys.argv[2]
e2e_dir = sys.argv[3]
scenario = sys.argv[4]
provider = sys.argv[5]
timebox_s = int(sys.argv[6])
allow_stub = int(sys.argv[7])
no_agents = int(sys.argv[8])
keep_repo = int(sys.argv[9])
bin_path = sys.argv[10]
stub_bin = sys.argv[11]

def cmd_out(argv):
    try:
        p = subprocess.run(argv, cwd=root, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=3, check=False)
        s = p.stdout.decode("utf-8", "replace").strip()
        return s if s else None
    except Exception:
        return None

git_head = cmd_out(["git", "rev-parse", "HEAD"])
git_status = cmd_out(["git", "status", "--porcelain"])

def version_of(tool):
    if shutil.which(tool) is None:
        return None
    # Best-effort; versions differ across tools.
    return cmd_out([tool, "--version"])

meta = {
    "scenario": scenario,
    "provider": provider,
    "timebox_s": timebox_s,
    "allow_stub": bool(allow_stub),
    "no_agents": bool(no_agents),
    "keep_repo": bool(keep_repo),
    "bin": bin_path,
    "stub_bin": stub_bin,
    "root": root,
    "e2e_dir": e2e_dir,
    "git_head": git_head,
    "git_dirty": bool(git_status),
    "versions": {
        "python3": cmd_out(["python3", "--version"]),
        "git": cmd_out(["git", "--version"]),
        "cargo": cmd_out(["cargo", "--version"]),
        "codex": version_of("codex"),
        "claude": version_of("claude"),
    },
}

with open(out_path, "w", encoding="utf-8") as f:
    json.dump(meta, f, indent=2, sort_keys=True)
    f.write("\n")
PY
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
  local stdin_file="${8:-}"

  local ts_ms
  ts_ms="$(now_ms)"

  printf '{' >>"$TRACE_PATH"
  printf '"ts_ms":%s' "$ts_ms" >>"$TRACE_PATH"
  printf ',"label":"%s"' "$(json_escape "$label")" >>"$TRACE_PATH"
  printf ',"cwd":"%s"' "$(json_escape "$cwd")" >>"$TRACE_PATH"
  printf ',"cmd":"%s"' "$(json_escape "$cmd")" >>"$TRACE_PATH"
  printf ',"args":%s' "$args_json" >>"$TRACE_PATH"
  printf ',"stdin_file":"%s"' "$(json_escape "$stdin_file")" >>"$TRACE_PATH"
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

  local stdin_file=""
  if [[ "${1:-}" == "--stdin-file" ]]; then
    stdin_file="${2:-}"
    shift 2
  fi

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
  python3 - "$timebox_s" "$cwd" "$stdout_file" "$stderr_file" "$stdin_file" "$cmd" "${args[@]}" <<'PY'
import os, subprocess, sys

timebox_s = int(sys.argv[1])
cwd = sys.argv[2]
stdout_path = sys.argv[3]
stderr_path = sys.argv[4]
stdin_path = sys.argv[5]
cmd = sys.argv[6:]

stdin_f = open(stdin_path, "rb") if stdin_path else None

with open(stdout_path, "wb") as out, open(stderr_path, "wb") as err:
    try:
        p = subprocess.run(cmd, cwd=cwd, stdin=stdin_f, stdout=out, stderr=err, timeout=timebox_s, check=False)
        sys.exit(p.returncode)
    except subprocess.TimeoutExpired:
        err.write(f"TIMEOUT after {timebox_s}s\n".encode("utf-8"))
        sys.exit(124)
    finally:
        if stdin_f is not None:
            stdin_f.close()
PY
  ec=$?
  set -e

  local stdout stderr
  stdout="$(cat "$stdout_file" 2>/dev/null || true)"
  stderr="$(cat "$stderr_file" 2>/dev/null || true)"

  trace_emit "$label" "$cwd" "$ec" "$cmd" "$args_json" "$stdout" "$stderr" "$stdin_file"
  return "$ec"
}

spawn_agent() {
  local label="$1"
  local repo="$2"
  local prompt_file="$3"
  local expect_token="${4:-SMOKE_AGENT_OK}"

  if [[ "$NO_AGENTS" -eq 1 ]]; then
    trace_emit "$label" "$repo" 0 "(skipped)" "[]" "" "" ""
    return 0
  fi

  if [[ ! -f "$prompt_file" ]]; then
    trace_emit "$label" "$repo" 2 "(missing prompt)" "[]" "" "missing prompt file: $prompt_file" ""
    fail "missing prompt file: $prompt_file"
  fi

  # Convention: run_with_timeout writes $OUT_DIR/$label.{stdout,stderr}.
  local stdout_file="$OUT_DIR/$label.stdout"
  local stderr_file="$OUT_DIR/$label.stderr"
  local ec=0

  case "$PROVIDER" in
    codex)
      command -v codex >/dev/null 2>&1 || fail "codex not found in PATH"
      run_with_timeout "$label" "$repo" "$TIMEBOX_S" \
        --stdin-file "$prompt_file" \
        codex exec --ephemeral --full-auto --json -C "$repo" -
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
      fail "unknown --provider: $PROVIDER (expected codex|claude|both)"
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

meta_patch_repo() {
  local repo="$1"
  python3 - "$META_PATH" "$OUT_DIR" "$TRACE_PATH" "$repo" "$KEEP_REPO" <<'PY'
import json, sys

meta_path = sys.argv[1]
out_dir = sys.argv[2]
trace_path = sys.argv[3]
repo_path = sys.argv[4]
keep_repo = bool(int(sys.argv[5]))

with open(meta_path, "r", encoding="utf-8") as f:
    meta = json.load(f)

meta["out_dir"] = out_dir
meta["trace_path"] = trace_path
meta["repo_path"] = repo_path
meta["keep_repo"] = keep_repo

with open(meta_path, "w", encoding="utf-8") as f:
    json.dump(meta, f, indent=2, sort_keys=True)
    f.write("\n")
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

# Hardening: in a claimed_by_other collision, action_plan should prompt the agent to read first.
plan = [str(x) for x in (v.get("action_plan") or [])]
if not any(" --agent-id B read" in x or x.endswith(" --agent-id B read") for x in plan):
    raise SystemExit(f"expected action_plan to include a read step for agent B; got: {plan!r}")
PY

  if [[ "$PROVIDER" == "both" ]]; then
    local saved_provider="$PROVIDER"
    local p
    for p in codex claude; do
      PROVIDER="$p"
      spawn_agent "agent.smoke.$p" "$repo" "$PROMPTS_DIR/smoke.$p.txt"
    done
    PROVIDER="$saved_provider"
  else
    spawn_agent "agent.smoke" "$repo" "$PROMPTS_DIR/smoke.${PROVIDER}.txt"
  fi
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

scenario_triangle() {
  local repo="$1"

  if [[ "$NO_AGENTS" -ne 1 ]] && [[ "$PROVIDER" != "codex" ]]; then
    fail "scenario triangle requires --provider codex (or pass --no-agents for a CLI-only run)"
  fi

  # Copy the CLI into the temp repo so agent prompts can use a stable relative path.
  cp "$BIN" "$repo/but-engineering-rewrite"
  chmod +x "$repo/but-engineering-rewrite"

  # Mirror harness Case 05i:
  # - Triangle claim conflict (A/B/C overlap).
  # - Dependency hints: B's intent overlaps A's API declaration (same scope) => exactly one hint for A.
  # - Noise control: C uses same token in different scope and must not influence B; C must not receive hints.
  run_with_timeout "cli.post.decl.A1" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id A post --type declaration --json '{"scope":"component:sync","tags":["component/api","provider"],"surface":["SyncService::push","SyncService::pull"],"note":"First declaration"}'
  run_with_timeout "cli.post.decl.A2" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id A post --type declaration --json '{"scope":"component:sync","tags":["component/api","provider"],"surface":["SyncService::push"],"note":"Second declaration (should dedupe)"}'

  run_with_timeout "cli.post.intent.B" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id B post --type intent --json '{"scope":"component:sync","tags":["consumer"],"surface":["SyncService::push"],"note":"Consume push"}'

  run_with_timeout "cli.post.intent.C" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id C post --type intent --json '{"scope":"component:auth","tags":["consumer"],"surface":["SyncService::push"],"note":"Same token name, different scope"}'
  run_with_timeout "cli.post.decl.C" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id C post --type declaration --json '{"scope":"component:auth","tags":["component/api","provider"],"surface":["SyncService::push"],"note":"Overlapping token, different scope; must not hint sync consumers"}'

  run_with_timeout "cli.claim.A" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id A claim --path src/app.txt --ttl 15m
  run_with_timeout "cli.claim.B" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id B claim --path src/ --ttl 15m
  run_with_timeout "cli.claim.C" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id C claim --path src/app.txt --ttl 15m

  run_with_timeout "cli.check.B.warn" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id B check --path src/app.txt
  run_with_timeout "cli.check.B.deny" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id B check --path src/app.txt --strict

  python3 - "$OUT_DIR/cli.check.B.warn.stdout" "$OUT_DIR/cli.check.B.deny.stdout" <<'PY'
import json, sys

warn_path, deny_path = sys.argv[1], sys.argv[2]
warn = json.load(open(warn_path, "r", encoding="utf-8"))
deny = json.load(open(deny_path, "r", encoding="utf-8"))

def ensure_blockers(v):
    blockers = v.get("blocking_agents") or []
    if not isinstance(blockers, list):
        raise SystemExit("expected blocking_agents to be a list")
    if "A" not in blockers or "C" not in blockers:
        raise SystemExit(f"expected blocking_agents to include A and C; got {blockers!r}")

    plan = v.get("action_plan") or []
    if not isinstance(plan, list) or not plan:
        raise SystemExit("expected non-empty action_plan list")
    s = "\n".join(str(x) for x in plan)
    if "@A" not in s or "@C" not in s:
        raise SystemExit("expected action_plan to mention both @A and @C")

def ensure_dependency_hints(v):
    hints = v.get("dependency_hints")
    if not isinstance(hints, list):
        raise SystemExit("expected dependency_hints to be a list")
    providers = [h.get("provider_agent_id") for h in hints if isinstance(h, dict)]
    if providers.count("A") != 1:
        raise SystemExit(f"expected exactly one dependency hint for provider A; got providers={providers!r}")
    if "C" in providers:
        raise SystemExit("did not expect a dependency hint for provider C (scope mismatch)")

    # Sanity: ensure the overlap token we expect is present.
    hint_a = next((h for h in hints if isinstance(h, dict) and h.get("provider_agent_id") == "A"), None)
    if not hint_a:
        raise SystemExit("missing dependency hint for provider A")
    toks = hint_a.get("overlap_tokens") or []
    if "SyncService::push" not in toks:
        raise SystemExit(f"expected overlap_tokens to include SyncService::push; got {toks!r}")

if warn.get("decision") != "warn":
    raise SystemExit(f"expected warn decision=warn; got {warn.get('decision')!r}")
if deny.get("decision") != "deny":
    raise SystemExit(f"expected strict decision=deny; got {deny.get('decision')!r}")

ensure_blockers(warn)
ensure_blockers(deny)
ensure_dependency_hints(warn)
ensure_dependency_hints(deny)
PY

  run_with_timeout "cli.check.A.warn" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id A check --path src/app.txt
  run_with_timeout "cli.check.C.warn" "$repo" "$TIMEBOX_S" \
    "$repo/but-engineering-rewrite" --agent-id C check --path src/app.txt

  python3 - "$OUT_DIR/cli.check.A.warn.stdout" "$OUT_DIR/cli.check.C.warn.stdout" <<'PY'
import json, sys

a_path, c_path = sys.argv[1], sys.argv[2]
a = json.load(open(a_path, "r", encoding="utf-8"))
c = json.load(open(c_path, "r", encoding="utf-8"))

if a.get("decision") != "warn":
    raise SystemExit(f"expected A decision=warn; got {a.get('decision')!r}")
if c.get("decision") != "warn":
    raise SystemExit(f"expected C decision=warn; got {c.get('decision')!r}")

for who, v, expected in [("A", a, ["B", "C"]), ("C", c, ["A", "B"])]:
    blockers = v.get("blocking_agents") or []
    if not isinstance(blockers, list):
        raise SystemExit(f"expected {who} blocking_agents list")
    for e in expected:
        if e not in blockers:
            raise SystemExit(f"expected {who} blocking_agents to include {e}; got {blockers!r}")

# C must NOT receive dependency hints about A's sync declaration (scope mismatch).
hints = c.get("dependency_hints")
if not isinstance(hints, list):
    raise SystemExit("expected C dependency_hints list")
if hints:
    providers = [h.get("provider_agent_id") for h in hints if isinstance(h, dict)]
    raise SystemExit(f"expected C to have no dependency hints; got providers={providers!r}")
PY

  if [[ "$NO_AGENTS" -ne 1 ]]; then
    spawn_agent "agent.triangle.step1" "$repo" "$PROMPTS_DIR/triangle.step1.${PROVIDER}.txt" "E2E_TRIANGLE_STEP1_OK"
  fi
}

main() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --scenario) SCENARIO="${2:-}"; shift 2 ;;
      --provider) PROVIDER="${2:-}"; shift 2 ;;
      --timebox-s) TIMEBOX_S="${2:-}"; shift 2 ;;
      --out-dir) OUT_DIR_OVERRIDE="${2:-}"; shift 2 ;;
      --allow-stub) ALLOW_STUB=1; shift 1 ;;
      --no-agents) NO_AGENTS=1; shift 1 ;;
      --keep-repo) KEEP_REPO=1; shift 1 ;;
      -h|--help) usage; exit 0 ;;
      *) fail "unknown arg: $1 (try --help)" ;;
    esac
  done

  case "$SCENARIO" in
    smoke|collision|discovery|triangle) ;;
    "" ) fail "--scenario requires a value (try --help)" ;;
    * ) fail "unknown --scenario: $SCENARIO (supported: smoke|collision|discovery|triangle)" ;;
  esac

  case "$PROVIDER" in
    codex|claude|both) ;;
    "" ) fail "--provider requires a value (try --help)" ;;
    * ) fail "unknown --provider: $PROVIDER (supported: codex|claude|both)" ;;
  esac

  if ! [[ "$TIMEBOX_S" =~ ^[0-9]+$ ]] || [[ "$TIMEBOX_S" -le 0 ]]; then
    fail "--timebox-s must be a positive integer (got: $TIMEBOX_S)"
  fi

  if [[ -n "$OUT_DIR_OVERRIDE" ]] && [[ "$OUT_DIR_OVERRIDE" == "." ]]; then
    fail "--out-dir must not be '.' (would pollute the repo); pick a subdir like e2e/out/custom"
  fi

  mk_out_dir
  bold "E2E out dir: $OUT_DIR"
  bold "E2E trace: $TRACE_PATH"

  ensure_bin

  if [[ "$PROVIDER" == "both" ]] && [[ "$SCENARIO" != "smoke" ]]; then
    fail "--provider both is only supported for --scenario smoke"
  fi

  local repo
  repo="$(mk_repo)"
  meta_patch_repo "$repo"
  bold "E2E repo: $repo"

set +e
case "$SCENARIO" in
  smoke) scenario_smoke "$repo" ;;
  collision) scenario_collision "$repo" ;;
  discovery) scenario_discovery "$repo" ;;
  triangle) scenario_triangle "$repo" ;;
  *)
    printf "Unknown scenario: %s\n" "$SCENARIO" >&2
    printf "Supported: smoke, collision, discovery, triangle\n" >&2
    exit 2
    ;;
esac
local ec=$?
set -e

  # Deterministic artifact so CI/local runs can consume results without parsing stdout.
  python3 - "$OUT_DIR/verdict.json" "$SCENARIO" "$PROVIDER" "$TIMEBOX_S" "$ec" "$TRACE_PATH" "$repo" "$KEEP_REPO" <<'PY'
import json, sys

out_path = sys.argv[1]
scenario = sys.argv[2]
provider = sys.argv[3]
timebox_s = int(sys.argv[4])
exit_code = int(sys.argv[5])
trace_path = sys.argv[6]
repo_path = sys.argv[7]
keep_repo = bool(int(sys.argv[8]))

v = {
    "pass": exit_code == 0,
    "exit_code": exit_code,
    "scenario": scenario,
    "provider": provider,
    "timebox_s": timebox_s,
    "trace_path": trace_path,
    "repo_path": repo_path,
    "keep_repo": keep_repo,
}

with open(out_path, "w", encoding="utf-8") as f:
    json.dump(v, f, indent=2, sort_keys=True)
    f.write("\n")
PY

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
