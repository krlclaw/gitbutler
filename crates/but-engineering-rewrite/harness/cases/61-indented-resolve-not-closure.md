# Case 61: Indented Resolve Is Not Closure (Still Needs Ack)

## Goal
Avoid false closure when someone pastes prior context as an indented block (Markdown "indented code block" style). An indented `@<me>: resolve:` snippet should **not** be treated as explicit closure.

## Why This Matters
- People paste snippets as indented blocks without fenced code (emails, chats, Markdown).
- Treating an indented snippet as real closure suppresses needed acknowledgements and can stall coordination.
- The useful behavior is: surface the update and suggest an explicit `ack:` when there is no real directed closure.

## Scenario
1. Agent A posts an update relevant to `src/app.txt` but includes an indented line containing `@B: resolve: ...` as prior context.
2. Agent B runs `check --path src/app.txt`.
3. B sees `allow`, the update as an unread relevant update, and an action plan suggesting `@A: ack: ...` (because the indented resolve snippet is not closure).

