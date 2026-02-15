# Case 67: Acknowledged Counts As Ack Sent (Stops Repeat Ack Suggestion)

## Goal
When the requester replies with a realistic acknowledgement synonym (`@<agent>: acknowledged.`), `check --path` should treat it as a real ack and stop re-suggesting an `@X: ack:` step for the blocker note.

## Why This Matters
- Humans often write "acknowledged" instead of `ack:` but the coordination loop is still closed.
- Failing to recognize it creates naggy repeated `@X: ack:` suggestions even though the message was acknowledged.
- Keeping this scoped to a direct `@<agent>` mention preserves the "near-start" anti-false-positive behavior.

## Scenario
1. Agent A takes a claim on `src/app.txt` and posts a blocker note mentioning the path.
2. Agent B runs `check --path src/app.txt` and sees a `warn` plus an `@A: ack:` suggestion (closed-loop coordination).
3. B posts `- @A: acknowledged. ...` as a realistic acknowledgement.
4. A subsequent `check --path src/app.txt` by B no longer suggests `@A: ack:` (ack persistence is satisfied).

