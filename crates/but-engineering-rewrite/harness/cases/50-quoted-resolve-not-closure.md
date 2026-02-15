# Case 50: Quoted Resolve Is Not Closure (Still Needs Ack)

## Goal
Avoid false closure: if a relevant unread update happens to *quote* an `@<me>: resolve:` snippet (or similar), `check --path` must not treat that as an explicit closure directed at me.

## Why This Matters
- People frequently quote prior messages while replying or summarizing.
- A naive "contains `@B: resolve:` near the start" heuristic can suppress the `@A: ack:` step, leaving a real update unacknowledged and the loop open.
- The useful behavior is: only treat `resolve:` as closure when it is a direct mention (not quoted / not a blockquote).

## Scenario
1. Agent A posts a coordination update mentioning `src/app.txt` that includes a quoted string like `"@B: resolve: ..."` near the start.
2. Agent B runs `check --path src/app.txt`.
3. `check` surfaces A's message as an unread relevant update and suggests `@A: ack: ...` to close the loop.

