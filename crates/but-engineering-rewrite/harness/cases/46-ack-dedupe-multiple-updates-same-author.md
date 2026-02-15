# Case 46: Ack Dedupe (Multiple Updates, Same Author)

## Goal
If a single agent posts multiple relevant coordination updates about the same path, `check --path` should suggest exactly one `@X: ack: ...` step for that author (not one per message).

## Why This Matters
- People often send follow-ups or split a thought across multiple messages.
- Recommending multiple acks is noisy and creates unnecessary coordination work.
- One explicit acknowledgement closes the loop without spamming the channel.

## Scenario
1. Agent A posts two messages mentioning `src/app.txt`.
2. Agent B runs `check --path src/app.txt`.
3. `check` surfaces both messages as unread relevant updates, but suggests only a single `@A: ack: ...` step.
4. A second `check` by B does not repeat the same unread updates nor the ack suggestion (cursor advanced).

