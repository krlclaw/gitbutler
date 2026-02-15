# Case 45: Blocker Note Suppresses Redundant Ping (Ack Instead)

## Goal
If the blocking agent has already posted a relevant coordination update about the path, `check --path` should not suggest the redundant generic ping (`Are you working on it?`). Instead it should surface the update and suggest an explicit `@X: ack:` step to close the loop.

## Why This Matters
- The default conflict ping is useful when the blocker is silent, but noisy when they've already communicated.
- In practice, blockers often post proactive notes ("I'm on it, will release soon"), and the best follow-up is acknowledgement, not another question.
- This keeps coordination low-friction: update -> surfaced -> ack, even under contention.

## Scenario
1. Agent A claims `src/app.txt`.
2. A posts a note mentioning `src/app.txt` (timeline/ownership update).
3. Agent B runs `check --path src/app.txt` and is warned due to A's claim.
4. `check` surfaces A's note as an unread relevant update, suggests `@A: ack: ...`, and does **not** suggest the generic `Are you working on it?` ping.

