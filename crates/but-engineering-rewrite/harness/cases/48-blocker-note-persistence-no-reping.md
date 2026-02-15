# Case 48: Blocker Note Persistence (Don't Re-Ping After Read)

## Goal
Ensure `check --path` does not re-suggest the generic conflict ping once the blocker has already communicated about the path, even after the update has been surfaced (cursor advanced). Keep the loop closed by continuing to suggest an explicit `ack:` until the checker actually acknowledges.

## Why This Matters
- In real teams, blockers often post proactive notes ("I'm on it, ETA ...").
- A checker may poll `check --path` multiple times; re-suggesting `Are you working on it?` after they already saw the blocker note creates spam and friction.
- Persisting an `ack:` suggestion until it is sent keeps coordination closed-loop rather than "open" and noisy.

## Scenario
1. Agent A claims `src/app.txt`.
2. A posts a note mentioning `src/app.txt` (ownership/ETA update).
3. Agent B runs `check --path src/app.txt`:
   - decision is `warn`
   - A's note is surfaced as a relevant update
   - action plan suggests `@A: ack: ...`
   - action plan does **not** suggest the generic `Are you working on it?` ping
4. Agent B runs `check --path src/app.txt` again (no new messages):
   - still `warn`
   - still does **not** suggest the generic ping
   - still suggests `@A: ack: ...` (until B actually acks)
5. B posts the suggested `@A: ack: ...`, then a third `check` should not repeat the ack suggestion.

