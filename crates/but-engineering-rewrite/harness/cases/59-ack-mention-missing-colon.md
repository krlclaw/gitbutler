# Case 59: Ack @Mention Missing Colon (Stops Repeat Ack Suggestion)

## Goal
Treat a common informal ack format (`@A ack: ...`) as an explicit acknowledgement so `check --path`
does not keep suggesting redundant `@A: ack:` actions.

## Why This Matters
- In real chat, people frequently omit the colon after an @mention.
- If the system fails to recognize that as an ack, it will keep nagging with the same closed-loop step.
- Keeping the heuristic near-start and directive-shaped preserves low-noise behavior.

## Scenario
1. Agent A claims `src/app.txt` and posts a blocker note.
2. Agent B runs `check --path src/app.txt` and is suggested to `@A: ack: ...`.
3. B posts a checklist-style ack but omits the colon: `- @A ack: ...`.
4. A subsequent `check` should not repeat the ack suggestion.

