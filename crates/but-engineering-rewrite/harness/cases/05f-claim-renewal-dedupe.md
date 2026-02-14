# Case 05f: Claim Renewal (No Duplicate Claims Rows)

## Motivation
Claims are leases with TTL. Agents will often "renew" a claim by re-claiming the same path while they continue work.

Renewal should not create duplicate rows for the same `(agent_id, path)` because it makes visibility (`claims`) noisy and can lead to confusing downstream behavior.

## Scenario
1. Agent A claims `src/app.txt`.
2. Agent A claims `src/app.txt` again (renewal).
3. Agent B runs `claims`.

## Expected
- `claims` contains exactly one claim row for `{"agent_id":"A","path":"src/app.txt",...}`.
