# Case 41: Miscommunication Repair (Release + Ack Closure)

## Goal
Exercise a realistic coordination loop where a broad claim blocks unrelated work, the misunderstanding is repaired via messages, and the loop is closed with an explicit acknowledgement.

## Why This Matters
- Broad claims happen (defensive directory holds) but can accidentally block teammates.
- The fastest path to unblocking is usually: clarify intent -> narrow/release -> confirm -> acknowledge.
- A closed loop (repair message surfaced -> ack) prevents repeated pings and helps teams converge.

## Scenario
1. Agent A takes a broad claim on `src/`.
2. Agent B runs `check --path src/app.txt` and is warned; the action plan suggests pinging A.
3. B posts a clarification request to A.
4. A replies acknowledging the request and releases the broad claim.
5. B re-runs `check --path src/app.txt` and sees `allow`, the repair message as an unread relevant update, and an action plan step to `@A: ack: ...`.
6. A subsequent `check` does not repeat the same update nor the ack suggestion.

