# Case 68: Resolved Without Punctuation Is Closure (No Ack Needed)

## Goal
Treat a bare `@<me>: resolved ...` (no trailing `:` or sentence punctuation) as explicit closure, so `check --path` does not suggest an `@X: ack:` back and create ack ping-pong noise.

## Why This Matters
- In real coordination threads, people often write `@B: resolved thanks` or `@B: resolve this is handled` without adding `:`/`.`/`!`.
- If we only accept `resolved:` / `resolved.` variants, the system will keep suggesting a redundant ack even though the blocker already closed the loop.

## Scenario
1. Agent A blocks `src/app.txt` with a broad `src/` claim.
2. Agent B pings A.
3. Agent A replies with a bare `@B: resolved ...` and releases the claim.
4. Agent B runs `check --path src/app.txt`.
5. `check` surfaces A's message once but does **not** suggest `@A: ack:` because the message is explicit closure.

