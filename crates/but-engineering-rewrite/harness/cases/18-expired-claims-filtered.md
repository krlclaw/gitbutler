# Case 18: Claims Listing (Filters Expired)

## Why
Coordination UIs often start with `claims` as the "who is editing what" source of truth.
If expired leases show up in `claims`, teams waste time pinging the wrong agent or avoid
touching safe paths.

## Scenario
1. Agent A claims `src/app.txt` with a very short TTL (1s).
2. Wait for the lease to expire.
3. Agent B runs `claims`.

## Expected
- The `claims` output does not include the expired `(agent_id=A, path=src/app.txt)` row.

