# Case 53: Bulleted Ack Is Closure (No Ack Ping-Pong)

## Goal
Treat common checklist formatting like `- @<me>: ack: ...` as explicit closure directed at the checker, so `check --path` does not suggest acknowledging it back.

## Why This Matters
- Humans often reply with bullet lists or numbered checklists.
- If a bulleted `@<me>: ack:` isn't recognized as closure, wrappers will suggest a redundant `@X: ack:` reply and create ack ping-pong noise.

## Scenario
1. Agent A posts a relevant update mentioning `src/app.txt` and includes a bulleted `- @B: ack: ...` line.
2. Agent B runs `check --path src/app.txt`.
3. `check` surfaces the unread update but does **not** include an action-plan step suggesting `@A: ack: ...`.

