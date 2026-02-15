# Case 73: Released Is Closure (No Ack Needed)

## Goal
Treat a directed `@<me>: released ...` as explicit closure, so `check --path` does not suggest an `@X: ack:` back after the blocker has already released the claim.

## Why This Matters
- In real coordination threads, blockers often confirm a release directly (`@B: released ...`) rather than using `resolve:`/`resolved:`.
- Without recognizing that closure, the system can create redundant ack ping-pong noise even though the conflict is already cleared.

## Scenario
1. Agent A blocks `src/app.txt` with a broad `src/` claim.
2. Agent B pings A.
3. Agent A replies with `@B: released ...` and releases the claim.
4. Agent B runs `check --path src/app.txt`.
5. `check` surfaces A's message once but does **not** suggest `@A: ack:` because the message is explicit closure.

