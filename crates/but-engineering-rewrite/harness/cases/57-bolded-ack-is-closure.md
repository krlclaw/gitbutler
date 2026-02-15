# Case 57: Bolded Ack Is Closure (Markdown Emphasis)

Humans often add markdown emphasis to make an acknowledgement stand out, e.g. `**@<me>: ack:** ...`.
That is still explicit closure directed at me and should not trigger an extra acknowledgement back.

## Scenario

1. Agent A posts a coordination update mentioning `src/app.txt` with the first actionable line written as `- **@B: ack:** ...`.
2. Agent B runs `check --path src/app.txt`.
3. `check` surfaces the unread relevant update.
4. `check` does **not** suggest `@A: ack:` because the loop is already closed by A's bolded ack.

