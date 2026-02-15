# Case 66: Resolved Question Mark Is Not Closure (Still Needs Ack)

## Goal
Avoid false closure: `@<me>: resolved?` is frequently a question ("is this resolved?"), not an explicit loop-closure directive like `@<me>: resolved:` or `@<me>: resolved.`.

## Why This Matters
- Treating questions as closure suppresses the `@X: ack:` suggestion and leaves the loop open.
- In practice, people often append `?` when asking for confirmation or handing off, even if they intend to be helpful.

## Scenario
1. Agent A takes a broad claim on `src/`.
2. Agent B runs `check --path src/app.txt` and is warned; the action plan suggests pinging A.
3. B posts a clarification question to A.
4. A replies with `@B: resolved? ...` and releases the broad claim.
5. B re-runs `check --path src/app.txt` and sees `allow` plus A's message as an unread relevant update.
6. `check` suggests an `@A: ack:` (because `resolved?` is not explicit closure).

