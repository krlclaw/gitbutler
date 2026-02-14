# Case 27: Check Ignores Expired Non-Blocking Claims

## Goal
When running `check`, the output should avoid stale coordination noise:

- If another agent had a **non-blocking claim** that has already **expired**, `check` should not include FYI steps for that agent.

This keeps `action_plan_by_agent` focused on actionable, current coordination.

## Harness Steps
1. A: `claim --path src/app.txt --ttl 15m` (blocking)
2. C: `claim --path src/other.txt --ttl 1s` (non-blocking, short-lived)
3. Wait for C's claim to expire.
4. B: `check --path src/app.txt`

## Expected
- Decision is `warn` (blocked by A).
- Output does not mention `C` or `src/other.txt` (expired non-blocking claim is ignored).

