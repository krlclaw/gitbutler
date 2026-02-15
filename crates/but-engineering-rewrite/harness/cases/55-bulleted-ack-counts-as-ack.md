# Case 55: Bulleted Ack Counts As Ack Sent (Stops Repeat Ack Suggestion)

## Goal
If the requester acknowledges a blocker using common checklist formatting like `- @<agent>: ack: ...`,
`check --path` should treat that as a real ack and stop suggesting another `@<agent>: ack:` on subsequent polls.

## Why This Matters
- People often reply to several threads in one message using bullets.
- If bullet-style acks don't count, the tool will keep prompting redundant acknowledgements and create noise.

## Scenario
1. Agent A holds a claim on `src/app.txt` and posts a relevant blocker note about that file.
2. Agent B runs `check --path src/app.txt` and is prompted to post `@A: ack: ...`.
3. B posts a reply containing a bulleted `- @A: ack: ...` line.
4. B re-runs `check --path src/app.txt`.

## Expected
- The second `check` does **not** suggest `@A: ack:` again (the loop is closed).

