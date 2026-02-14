# Case 32: Allow Still Includes Non-Blocking FYI

## Goal
Even when `check` is **not blocked**, it should still surface low-noise coordination context:

- If another agent has an active claim on a different path (non-blocking), `check` should include a useful FYI step for that agent in `action_plan_by_agent`.
- The non-blocking agent should **not** be told to release their unrelated claim.

This helps teams avoid duplicating work without turning coordination into churn.

## Harness Steps
1. A: `claim --path docs/readme.md --ttl 15m` (non-blocking)
2. B: `check --path src/app.txt` (no blockers)

## Expected
- Decision is `allow`.
- `action_plan_by_agent["A"]` includes an FYI message mentioning `docs/readme.md` and `src/app.txt`.
- `action_plan_by_agent["A"]` does **not** suggest `release --path ...`.

