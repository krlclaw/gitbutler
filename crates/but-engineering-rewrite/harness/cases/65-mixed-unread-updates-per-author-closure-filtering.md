# Case 65: Mixed Unread Updates (Per-Author Closure Filtering)

## Goal
When `check --path` surfaces multiple unread relevant updates, only suggest `@X: ack:` for authors whose updates still require closed-loop acknowledgement. If one unread update is explicit closure directed at the requester (e.g. `@<me>: resolve:`), do not suggest acking that author back.

## Why This Matters
- Coordination often involves multiple agents commenting on the same file.
- If any one update contains explicit closure, it should not suppress acks for other authors who still need acknowledgement.
- Conversely, suggesting acks to an author who already explicitly closed the loop (`resolve:`) creates avoidable ping-pong noise.

## Scenario
1. Agent A posts an update that includes `@B: resolve: ...` for `src/app.txt`.
2. Agent C posts a separate FYI update about `src/app.txt`.
3. Agent B runs `check --path src/app.txt`.
4. `check` surfaces both unread updates, suggests acking C, and does **not** suggest acking A.

