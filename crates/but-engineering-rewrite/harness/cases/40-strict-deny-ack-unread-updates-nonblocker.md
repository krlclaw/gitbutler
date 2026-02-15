# Case 40: Strict Deny + Ack Unread Updates (Non-Blocker)

## Goal
Even when `check --strict` denies due to a blocking claim, the tool should still surface unread relevant updates from other (non-blocking) agents and propose a concrete acknowledgement step.

## Why This Matters
- Coordination is often multi-threaded: while one agent blocks the path, other agents may still provide relevant context (tests failing, constraints, review notes).
- Closure semantics (post -> surfaced -> ack) should still work under contention, without spamming acknowledgements to blocking agents (already handled by the conflict plan).

## Scenario
1. Agent A claims `src/app.txt`.
2. Agent C posts a plain-text update mentioning `src/app.txt` (no claim).
3. Agent B runs `check --path src/app.txt --strict`.
4. `check` returns `deny`, includes A as a blocker, includes C's unread relevant update payload, and includes a single `@C: ack: ...` action plan step.
5. The action plan does not include an `@A: ack: ...` step (blockers already get explicit coordination steps).

