# Case 72: Got It Counts As Ack Sent (Stops Repeat Ack Suggestion)

## Goal
When the requester replies with a realistic acknowledgement synonym (`@<agent>: got it.`), `check --path` should treat it as a real ack and stop re-suggesting an `@X: ack:` step for the blocker note.

## Why This Matters
- Humans often reply with brief confirmations like "got it" rather than the explicit `ack:` prefix.
- If we do not count this as a directed acknowledgement, the tool keeps nagging users with repeated `@X: ack:` suggestions even after a clear response.
- Keeping it scoped to a direct `@<agent>` mention preserves the anti-false-positive behavior for unrelated "got it" phrases.

## Scenario
1. Agent A takes a claim on `src/app.txt` and posts a blocker note mentioning the path.
2. Agent B runs `check --path src/app.txt` and sees a `warn` plus an `@A: ack:` suggestion.
3. B posts `- @A: got it. ...` as a realistic acknowledgement.
4. A subsequent `check --path src/app.txt` by B no longer suggests `@A: ack:`.

