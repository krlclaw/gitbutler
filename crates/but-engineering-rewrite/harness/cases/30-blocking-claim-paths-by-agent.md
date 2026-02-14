# Case 30: Blocking Claim Paths By Agent (Mapping)

## Why
The `check` output already includes `blocking_claims` as a flat list of `{agent_id, path, expires_at_ms}`.
That is sufficient, but wrappers typically need to group those paths per blocking agent to render an
actionable UI (for example: "A is blocking you via: src/app.txt, src/").

This case locks in a small coordination-usefulness improvement:
- `check` should also return a `blocking_claim_paths_by_agent` mapping, so clients can display the
  relevant paths per blocker without having to group the flat list themselves.

## Harness Steps
1. A: `claim --path src/ --ttl 15m`
2. A: `claim --path src/app.txt --ttl 15m`
3. B: `check --path src/app.txt`

## Expected
- `check` returns `warn` with blocker `A`.
- `blocking_claim_paths_by_agent.A` exists and lists both overlapping paths.
- The list is ordered most-specific first (e.g. `src/app.txt` before `src`).
