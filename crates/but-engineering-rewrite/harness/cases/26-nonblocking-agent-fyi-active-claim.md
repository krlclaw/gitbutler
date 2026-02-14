# Case 26: Non-Blocking Agent FYI (Active Claim)

## Goal
When running `check`, the output should help coordinate beyond just listing blockers:

- If another agent has an **active claim on a different path** (non-blocking), `check` should still include a useful FYI step for that agent in `action_plan_by_agent`.
- The non-blocking agent should **not** be told to release their unrelated claim.

This makes multi-agent coordination more actionable by surfacing "who is doing what" while avoiding unnecessary churn.

## Harness Steps
1. A: `claim --path src/app.txt --ttl 15m` (blocking)
2. C: `claim --path src/other.txt --ttl 15m` (non-blocking)
3. B: `check --path src/app.txt`

## Expected
- Decision is `warn` (blocked by A).
- `action_plan_by_agent` includes an entry for `C` that says they are working on `src/other.txt` and not touching `src/app.txt`.
- `action_plan_by_agent` does **not** suggest `C` release `src/other.txt`.

