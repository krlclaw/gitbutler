# Case 58: Parenthesized Resolve Is Closure (No Ack Needed)

## Goal
Treat a common punctuation wrapper as still being explicit closure: `(@<me>: resolve: ...)` should close the loop just like `@<me>: resolve: ...`.

## Why This Matters
- Humans often wrap short directives in parentheses to reduce tone or add an aside.
- If `check --path` misses this and suggests `@X: ack:` anyway, it creates ack ping-pong noise even though the blocker already closed the loop explicitly.

## Scenario
1. Agent A takes a broad claim on `src/`.
2. Agent B runs `check --path src/app.txt` and is warned; the action plan suggests pinging A.
3. B posts a clarification question to A.
4. A replies with `(@B: resolve: ...)` and releases the broad claim.
5. B re-runs `check --path src/app.txt` and sees `allow` plus the resolve message as an unread relevant update.
6. `check` does **not** suggest `@A: ack:` for the parenthesized resolve.

