# Case 42: Ack Loop Suppression (Don't Ack the Auto-Ack)

## Goal
Prevent pointless ack ping-pong: when an agent posts the exact auto-ack message suggested by `check`, the recipient should not be prompted to acknowledge that auto-ack back.

## Why This Matters
- Closure semantics are good, but naive "always ack unread updates" can create infinite acknowledgement loops.
- The auto-ack template is meant to close the loop, not create new coordination work.
- Suppressing only the known auto-ack phrase keeps the heuristic narrow and avoids breaking real repair messages that happen to contain `ack:`.

## Scenario
1. Agent A posts a coordination note mentioning `src/app.txt`.
2. Agent B runs `check --path src/app.txt` and is allowed; the action plan suggests a `@A: ack: ...` step.
3. B posts the suggested auto-ack message: `@A: ack: saw your update re src/app.txt.`
4. Agent A runs `check --path src/app.txt` and sees B's ack as an unread relevant update.
5. `check` does not suggest an `@B: ack: ...` step in response to that auto-ack.

