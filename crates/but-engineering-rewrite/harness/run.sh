#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
BIN="${BIN:-$ROOT/target/debug/but-engineering-rewrite}"
STUB_BIN="$ROOT/crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite"

bold() { printf "\033[1m%s\033[0m\n" "$*"; }
fail() { printf "FAIL: %s\n" "$*" >&2; return 1; }

ensure_bin() {
  local default_bin="$ROOT/target/debug/but-engineering-rewrite"

  # If Rust isn't installed and we're using the default BIN path, prefer the
  # stub to avoid accidentally running a stale debug binary.
  if [[ "$BIN" == "$default_bin" ]] && ! command -v cargo >/dev/null 2>&1; then
    if [[ -x "$STUB_BIN" ]]; then
      BIN="$STUB_BIN"
      return 0
    fi
  fi

  # If we're using the default debug binary and cargo is available, rebuild
  # before running to avoid stale-binary issues while iterating on cases.
  if [[ "$BIN" == "$default_bin" ]] && command -v cargo >/dev/null 2>&1; then
    bold "Building but-engineering-rewrite..."
    (cd "$ROOT" && cargo build -p but-engineering-rewrite >/dev/null)
  fi

  if [[ -x "$BIN" ]]; then
    return 0
  fi

  # Allow running the harness without Rust installed.
  if [[ -x "$STUB_BIN" ]]; then
    BIN="$STUB_BIN"
    return 0
  fi

  if ! command -v cargo >/dev/null 2>&1; then
    fail "cargo not found and binary missing at $BIN (install Rust or set BIN=... to a prebuilt binary)"
  fi

  bold "Building but-engineering-rewrite..."
  (cd "$ROOT" && cargo build -p but-engineering-rewrite >/dev/null)

  [[ -x "$BIN" ]] || fail "binary not found at $BIN (set BIN=... to override)"
}

mk_repo() {
  local repo
  repo="$(mktemp -d "${TMPDIR:-/tmp}/but-erw-harness.XXXXXX")"
  git -C "$repo" init -q
  git -C "$repo" config user.email "harness@example.invalid"
  git -C "$repo" config user.name "Harness"
  mkdir -p "$repo/src"
  printf "hello\n" >"$repo/src/app.txt"
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "init"
  echo "$repo"
}

run_cli() {
  local repo="$1"
  local agent="$2"
  shift 2
  (cd "$repo" && "$BIN" --agent-id "$agent" "$@") 2>&1 || true
}

expect_contains() {
  local hay="$1"
  local needle="$2"
  if ! printf "%s" "$hay" | grep -Fq -- "$needle"; then
    printf '%s\n' "---- output ----" "$hay" "----------------" >&2
    fail "expected output to contain: $needle"
  fi
}

expect_matches() {
  local hay="$1"
  local re="$2"
  if ! printf "%s" "$hay" | grep -Eq -- "$re"; then
    printf '%s\n' "---- output ----" "$hay" "----------------" >&2
    fail "expected output to match regex: $re"
  fi
}

expect_not_contains() {
  local hay="$1"
  local needle="$2"
  if printf "%s" "$hay" | grep -Fq -- "$needle"; then
    printf '%s\n' "---- output ----" "$hay" "----------------" >&2
    fail "expected output to NOT contain: $needle"
  fi
}

case_01_two_agent_conflict() {
  bold "Case 01: Two-Agent Collision (Advisory)"
  local repo; repo="$(mk_repo)"
  trap '[[ -n "${repo:-}" ]] && rm -rf "$repo"' RETURN

  local out_a out_b
  out_a="$(run_cli "$repo" "A" claim --path src/app.txt --ttl 15m)"
  out_b="$(run_cli "$repo" "B" check --path src/app.txt)"

  # Expected behavior: B warned due to A's active intent.
  expect_contains "$out_b" "\"decision\""
  expect_contains "$out_b" "\"warn\""
  expect_contains "$out_b" "claimed_by_other"
}

case_01b_two_agent_conflict_strict() {
  bold "Case 01b: Two-Agent Collision (Strict Deny)"
  local repo; repo="$(mk_repo)"
  trap '[[ -n "${repo:-}" ]] && rm -rf "$repo"' RETURN

  local out_a out_b
  out_a="$(run_cli "$repo" "A" claim --path src/app.txt --ttl 15m)"
  out_b="$(run_cli "$repo" "B" check --path src/app.txt --strict)"

  expect_contains "$out_b" "\"decision\""
  expect_contains "$out_b" "\"deny\""
  expect_contains "$out_b" "claimed_by_other"
}

case_02_lease_expiry() {
  bold "Case 02: Lease Expiry"
  local repo; repo="$(mk_repo)"
  trap '[[ -n "${repo:-}" ]] && rm -rf "$repo"' RETURN

  local out_a out_b_claim out_b_check
  out_a="$(run_cli "$repo" "A" claim --path src/app.txt --ttl 2s)"
  sleep 3
  out_b_claim="$(run_cli "$repo" "B" claim --path src/app.txt --ttl 15m)"
  out_b_check="$(run_cli "$repo" "B" check --path src/app.txt)"

  # Expected future behavior: after TTL expiry, B can proceed.
  expect_contains "$out_b_check" "\"decision\""
  expect_contains "$out_b_check" "\"allow\""
  expect_contains "$out_b_check" "no_conflict"
}

case_03_habit_formation() {
  bold "Case 03: Habit Formation"
  local repo; repo="$(mk_repo)"
  trap '[[ -n "${repo:-}" ]] && rm -rf "$repo"' RETURN

  local baseline after
  baseline="$(run_cli "$repo" "A" eval user-prompt-submit)"

  run_cli "$repo" "A" post "I will edit src/app.txt" >/dev/null
  run_cli "$repo" "A" claim --path src/app.txt --ttl 15m >/dev/null

  after="$(run_cli "$repo" "B" eval user-prompt-submit)"

  # Expected future behavior: plain text nudge + live state.
  expect_contains "$baseline" "Announce what you'll do"
  expect_contains "$after" "claims"
}

case_04_high_signal_discovery() {
  bold "Case 04: High-Signal Discovery"
  local repo; repo="$(mk_repo)"
  trap '[[ -n "${repo:-}" ]] && rm -rf "$repo"' RETURN

  local out_post out_brief out_digest
  # Post a low-signal discovery that should NOT propagate into brief/digest by default.
  out_post="$(run_cli "$repo" "A" post --type discovery --json '{"signal":"low","title":"Minor note: update comment wording","evidence":[{"kind":"file","path":"src/app.txt","note":"comment style nit"}],"suggested_action":{"kind":"run","cmd":"echo ERW_CASE04_LOW_UNIQUE","note":"low-signal: should not surface"}}')"

  # Post a high-signal discovery that SHOULD propagate and drive next_steps.
  out_post="$(run_cli "$repo" "A" post --type discovery --json '{"signal":"high","title":"Lockfile indicates workspace mismatch","evidence":[{"kind":"file","path":"Cargo.lock","note":"has deps not in Cargo.toml"},{"kind":"cmd","cmd":"git status --porcelain","note":"repo is clean; mismatch is from lockfile"}],"suggested_action":{"kind":"run","cmd":"echo ERW_CASE04_HIGH_UNIQUE","note":"prove next_steps is derived from payload"}}')"
  out_brief="$(run_cli "$repo" "B" brief --type discovery)"
  out_digest="$(run_cli "$repo" "B" digest --type discovery)"

  expect_contains "$out_brief" "\"mode\":\"brief\""
  expect_contains "$out_brief" "\"discoveries\""
  expect_contains "$out_brief" "\"title\""
  expect_contains "$out_brief" "\"evidence\""
  expect_contains "$out_brief" "\"suggested_action\""
  expect_contains "$out_brief" "\"next_steps\""
  # Ensure next_steps is derived from the posted payload (not just echoed in discoveries).
  # `[^]]` is the portable way to match "anything but a closing bracket" across grep variants.
  expect_matches "$out_brief" "\"next_steps\"[[:space:]]*:[[:space:]]*\\[[^]]*ERW_CASE04_HIGH_UNIQUE"
  expect_not_contains "$out_brief" "ERW_CASE04_LOW_UNIQUE"

  expect_contains "$out_digest" "\"mode\":\"digest\""
  expect_contains "$out_digest" "\"discoveries\""
  expect_contains "$out_digest" "\"next_steps\""
  expect_matches "$out_digest" "\"next_steps\"[[:space:]]*:[[:space:]]*\\[[^]]*ERW_CASE04_HIGH_UNIQUE"
  expect_not_contains "$out_digest" "ERW_CASE04_LOW_UNIQUE"
}

case_05_dependency_hint() {
  bold "Case 05: Dependency Hint (Provider/Consumer API)"
  local repo; repo="$(mk_repo)"
  trap '[[ -n "${repo:-}" ]] && rm -rf "$repo"' RETURN

  local out_post out_check

  # Agent A declares an API surface (provider-side).
  out_post="$(run_cli "$repo" "A" post --type declaration --json '{"scope":"component:sync","tags":["component/api","provider"],"surface":["SyncService::push","SyncService::pull"],"note":"Refactor request/response types"}')"

  # Agent B declares intent that overlaps that API surface (consumer-side).
  out_post="$(run_cli "$repo" "B" post --type intent --json '{"scope":"component:sync","tags":["consumer"],"surface":["SyncService::push"],"note":"Implement CLI command that calls push"}')"

  # No hard locks: still allow, but emit a dependency hint.
  out_check="$(run_cli "$repo" "B" check --path src/app.txt)"

  expect_contains "$out_check" "\"decision\""
  expect_contains "$out_check" "\"allow\""
  expect_contains "$out_check" "\"dependency_hints\""
  expect_contains "$out_check" "\"provider_agent_id\":\"A\""
  expect_contains "$out_check" "SyncService::push"
  expect_contains "$out_check" "\"why\""
  expect_contains "$out_check" "\"next_step\""
}

case_05d_dependency_hint_scope_filter() {
  bold "Case 05d: Dependency Hint (Scope Filter)"
  local repo; repo="$(mk_repo)"
  trap '[[ -n "${repo:-}" ]] && rm -rf "$repo"' RETURN

  local out_post out_check

  # Agent A declares an API surface, but for a different scope.
  out_post="$(run_cli "$repo" "A" post --type declaration --json '{"scope":"component:auth","tags":["component/api","provider"],"surface":["SyncService::push"],"note":"Similar token name in another domain; should not trigger sync hints"}')"

  # Agent B declares intent for the sync scope.
  out_post="$(run_cli "$repo" "B" post --type intent --json '{"scope":"component:sync","tags":["consumer"],"surface":["SyncService::push"],"note":"Token overlaps, but scope differs; no hint expected"}')"

  out_check="$(run_cli "$repo" "B" check --path src/app.txt)"

  expect_contains "$out_check" "\"decision\""
  expect_contains "$out_check" "\"allow\""
  expect_contains "$out_check" "\"dependency_hints\""
  expect_matches "$out_check" "\"dependency_hints\"[[:space:]]*:[[:space:]]*\\[[[:space:]]*\\]"
  expect_not_contains "$out_check" "\"provider_agent_id\":\"A\""
}

case_05b_check_action_plan() {
  bold "Case 05b: Check Action Plan (Includes Blocking Agent)"
  local repo; repo="$(mk_repo)"
  trap '[[ -n "${repo:-}" ]] && rm -rf "$repo"' RETURN

  local out_a out_b
  out_a="$(run_cli "$repo" "A" claim --path src/app.txt --ttl 15m)"
  out_b="$(run_cli "$repo" "B" check --path src/app.txt)"

  expect_contains "$out_b" "\"decision\""
  expect_contains "$out_b" "\"warn\""
  expect_contains "$out_b" "\"blocking_agents\""
  expect_matches "$out_b" "\"blocking_agents\"[[:space:]]*:[[:space:]]*\\[[^]]*\"A\""
  expect_contains "$out_b" "\"action_plan\""
  # Action plan should be directly actionable and point at the blocking agent.
  expect_matches "$out_b" "\"action_plan\"[[:space:]]*:[[:space:]]*\\[[^]]*@A"
}

case_05c_dedup_blocking_agents() {
  bold "Case 05c: Dedupe Blocking Agents (No Duplicate @Agent IDs)"
  local repo; repo="$(mk_repo)"
  trap '[[ -n "${repo:-}" ]] && rm -rf "$repo"' RETURN

  local out_a1 out_a2 out_b

  # Simulate "renewal" / repeated claims: the check output should not list
  # the same blocking agent multiple times.
  out_a1="$(run_cli "$repo" "A" claim --path src/app.txt --ttl 15m)"
  out_a2="$(run_cli "$repo" "A" claim --path src/app.txt --ttl 15m)"
  out_b="$(run_cli "$repo" "B" check --path src/app.txt)"

  expect_contains "$out_b" "\"decision\""
  expect_contains "$out_b" "\"warn\""
  # Must be exactly one "A" (not ["A","A",...]).
  expect_matches "$out_b" "\"blocking_agents\"[[:space:]]*:[[:space:]]*\\[[[:space:]]*\"A\"[[:space:]]*\\]"
  expect_not_contains "$out_b" "\"A\",\"A\""
}

case_06_read_surfaces() {
  bold "Case 06: Inspect Surfaces (Read Declarations + Intents)"
  local repo; repo="$(mk_repo)"
  trap '[[ -n "${repo:-}" ]] && rm -rf "$repo"' RETURN

  local out_post out_read_decl out_read_intent

  out_post="$(run_cli "$repo" "A" post --type declaration --json '{"scope":"component:sync","tags":["component/api","provider"],"surface":["SyncService::push","SyncService::pull"],"note":"Refactor request/response types"}')"
  out_post="$(run_cli "$repo" "B" post --type intent --json '{"scope":"component:sync","tags":["consumer"],"surface":["SyncService::push"],"note":"Implement CLI command that calls push"}')"

  out_read_decl="$(run_cli "$repo" "B" read --type declaration)"
  out_read_intent="$(run_cli "$repo" "B" read --type intent)"

  expect_contains "$out_read_decl" "\"ok\""
  expect_contains "$out_read_decl" "\"kind\":\"declaration\""
  expect_contains "$out_read_decl" "\"messages\""
  expect_contains "$out_read_decl" "\"agent_id\":\"A\""
  expect_contains "$out_read_decl" "SyncService::pull"

  expect_contains "$out_read_intent" "\"ok\""
  expect_contains "$out_read_intent" "\"kind\":\"intent\""
  expect_contains "$out_read_intent" "\"messages\""
  expect_contains "$out_read_intent" "\"agent_id\":\"B\""
  expect_contains "$out_read_intent" "SyncService::push"
}

case_07_release_claim() {
  bold "Case 07: Release Claim (Explicit Unclaim)"
  local repo; repo="$(mk_repo)"
  trap '[[ -n "${repo:-}" ]] && rm -rf "$repo"' RETURN

  local out_a_claim out_b_check1 out_a_release out_b_check2

  out_a_claim="$(run_cli "$repo" "A" claim --path src/app.txt --ttl 15m)"
  out_b_check1="$(run_cli "$repo" "B" check --path src/app.txt)"

  # After an explicit release, others should no longer see a conflict.
  out_a_release="$(run_cli "$repo" "A" release --path src/app.txt)"
  out_b_check2="$(run_cli "$repo" "B" check --path src/app.txt)"

  expect_contains "$out_b_check1" "\"decision\""
  expect_contains "$out_b_check1" "\"warn\""
  expect_contains "$out_b_check1" "claimed_by_other"

  expect_contains "$out_a_release" "\"ok\""

  expect_contains "$out_b_check2" "\"decision\""
  expect_contains "$out_b_check2" "\"allow\""
  expect_contains "$out_b_check2" "no_conflict"
}

case_08_path_prefix_overlap() {
  bold "Case 08: Path Prefix Overlap (Directory Claim)"
  local repo; repo="$(mk_repo)"
  trap '[[ -n "${repo:-}" ]] && rm -rf "$repo"' RETURN

  local out_a out_b
  # Claim a directory, then check a file inside it. The CLI should treat
  # ancestor/descendant paths as overlapping.
  out_a="$(run_cli "$repo" "A" claim --path src/ --ttl 15m)"
  out_b="$(run_cli "$repo" "B" check --path src/app.txt)"

  expect_contains "$out_b" "\"decision\""
  expect_contains "$out_b" "\"warn\""
  expect_contains "$out_b" "claimed_by_other"
}

case_09_channel_messages() {
  bold "Case 09: Channel Messages (post/read transcript)"
  local repo; repo="$(mk_repo)"
  trap '[[ -n "${repo:-}" ]] && rm -rf "$repo"' RETURN

  local out_post_a out_post_b out_read
  out_post_a="$(run_cli "$repo" "A" post "@B: I'm going to refactor src/app.txt next")"
  out_post_b="$(run_cli "$repo" "B" post "@A: ack, I'm only reading for now")"
  out_read="$(run_cli "$repo" "B" read --type message)"

  expect_contains "$out_post_a" "\"ok\":true"
  expect_contains "$out_read" "\"ok\":true"
  expect_contains "$out_read" "\"kind\":\"message\""
  expect_contains "$out_read" "\"messages\""
  expect_contains "$out_read" "\"agent_id\":\"A\""
  expect_contains "$out_read" "refactor src/app.txt"
  expect_contains "$out_read" "@B"
}

main() {
  ensure_bin

  local failed=0
  case_01_two_agent_conflict || failed=1
  case_01b_two_agent_conflict_strict || failed=1
  case_02_lease_expiry || failed=1
  case_03_habit_formation || failed=1
  case_04_high_signal_discovery || failed=1
  case_05_dependency_hint || failed=1
  case_05d_dependency_hint_scope_filter || failed=1
  case_05b_check_action_plan || failed=1
  case_05c_dedup_blocking_agents || failed=1
  case_06_read_surfaces || failed=1
  case_07_release_claim || failed=1
  case_08_path_prefix_overlap || failed=1
  case_09_channel_messages || failed=1

  if [[ "$failed" -ne 0 ]]; then
    printf "\nOne or more cases failed. This is expected until the CLI is implemented.\n" >&2
    exit 1
  fi
}

main "$@"
