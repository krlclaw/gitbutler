# Case 49: Resolve Closes Loop (No Ack Needed)

## Goal
Exercise explicit closure semantics beyond `ack:`: when a blocker posts a `@<me>: resolve:` message, the coordination loop is already closed and `check --path` should not suggest acknowledging it back.

## Why This Matters
- Teams often use "resolve" to indicate the issue is handled and the other person can proceed.
- Suggesting an extra `ack:` after an explicit `resolve:` adds noise and can create needless ping-pong.
- The useful behavior is: surface the resolve note once, then let the requester move on.

## Scenario
1. Agent A takes a broad claim on `src/`.
2. Agent B runs `check --path src/app.txt` and is warned; the action plan suggests pinging A.
3. B posts a clarification question to A.
4. A replies with `@B: resolve: ...` and releases the broad claim.
5. B re-runs `check --path src/app.txt` and sees `allow` plus the resolve message as an unread relevant update.
6. `check` does **not** suggest `@A: ack:` for the resolve message, and subsequent polls do not repeat the same update.

