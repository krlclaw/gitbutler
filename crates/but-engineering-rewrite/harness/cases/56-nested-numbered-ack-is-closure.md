# Case 56: Nested Numbered Ack Is Closure (No Ack Ping-Pong)

Humans often write "reply checklists" that nest an ordered list inside a bullet list. An `@<me>: ack:` inside such a nested list is still explicit closure directed at me and should not trigger an extra acknowledgement back.

## Scenario

1. Agent A posts a coordination update mentioning `src/app.txt` where the first actionable line is written as `- 1. @B: ack: ...`.
2. Agent B runs `check --path src/app.txt`.
3. `check` surfaces the unread relevant update.
4. `check` does **not** suggest `@A: ack:` because the loop is already closed by A's ack.

