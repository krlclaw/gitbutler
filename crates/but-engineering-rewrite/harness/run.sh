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

main() {
  ensure_bin

  local failed=0
  case_01_two_agent_conflict || failed=1
  case_01b_two_agent_conflict_strict || failed=1
  case_02_lease_expiry || failed=1
  case_03_habit_formation || failed=1

  if [[ "$failed" -ne 0 ]]; then
    printf "\nOne or more cases failed. This is expected until the CLI is implemented.\n" >&2
    exit 1
  fi
}

main "$@"
