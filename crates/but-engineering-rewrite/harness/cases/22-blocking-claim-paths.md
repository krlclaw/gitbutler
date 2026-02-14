# Case 22: Check Includes Blocking Claim Paths

## Why
When `check` is blocked or warned, it's much more actionable if the output
includes the *specific claim path(s)* causing the overlap (not just agent ids).
This lets wrappers show "what exactly is blocked" and propose precise releases.

## Harness Steps
1. Agent A: `claim --path src/ --ttl 15m`
2. Agent B: `check --path src/app.txt`

## Expected
- `check` returns advisory collision (`decision:"warn"` + `claimed_by_other`).
- Output JSON includes a `blocking_claims` array with objects including:
  - `agent_id` of the blocker
  - `path` of the blocking claim (normalized, so `src/` becomes `src`)
  - `expires_at_ms` for ETA/urgency

