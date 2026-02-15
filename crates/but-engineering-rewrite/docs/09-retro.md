
## 2026-02-15: Drift Scenario Implemented & Passing

### What We Built
New E2E scenario `drift` (runtime nonce anti-gaming test):
- Runtime-generated random nonce (secrets.token_hex(8)) - NOT in agent prompt
- Random N claims (2-5) created by agent A
- Agent B must discover nonce via `brief` and post back `nonce=<value> claims: <N>`
- Assertion checks both nonce and claims count in B's message

### Test Results
✅ **Both modes passing:**
- `--no-agents` (deterministic CLI simulation): PASS
- `--provider codex` (real agent): PASS

Real agent run showed correct tool use:
1. eval → discovered claims count
2. brief → extracted nonce from discovery evidence
3. post → included both nonce and claims count

### Engineering Notes
- Validation requires `suggested_action.cmd` in discovery payloads (validate_discovery_payload)
- Python heredoc within bash: use `python3 - "$arg1" <<'PY'` to pass shell vars as sys.argv
- macOS sed requires explicit backup extension or empty string for in-place: `-i.bak` or `-i ""`

### Next
- Add more drift/distraction content to prompt (longer session simulation)
- Consider adding a timer/turn-count assertion to ensure agent doesn't loop
- Add scenario to CI once harness green across all 5 scenarios
