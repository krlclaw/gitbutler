# Case 60: Resolved With Punctuation Is Closure (No Ack Needed)

## Goal
Treat a common punctuation variant as explicit closure: `@<me>: resolved.` should close the loop just like `@<me>: resolved:`.

## Why This Matters
- Humans often write short directives with sentence punctuation (`.` / `!`) instead of a trailing colon.
- If `check --path` misses this, it suggests an `@X: ack:` back and creates unnecessary ack ping-pong even though the blocker already closed the loop.

## Scenario
1. Agent A takes a broad claim on `src/`.
2. Agent B runs `check --path src/app.txt` and is warned; the action plan suggests pinging A.
3. B posts a clarification question to A.
4. A replies with `@B: resolved.` and releases the broad claim.
5. B re-runs `check --path src/app.txt` and sees `allow` plus the message as an unread relevant update.
6. `check` does **not** suggest `@A: ack:` for the `resolved.` closure.

