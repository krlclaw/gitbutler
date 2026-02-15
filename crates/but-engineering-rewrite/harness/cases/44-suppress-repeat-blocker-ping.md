# Case 44: Suppress Repeat Blocker Ping (After Posted)

## Goal
When a checker is blocked and already followed the action plan by pinging the blocking agent, repeated `check --path` calls should not keep suggesting the same ping again until something changes.

## Why This Matters
- Repeated "ping the blocker" suggestions create coordination spam and can lead to duplicate @mentions.
- The useful loop is: detect conflict -> ping once -> wait/read -> retry, not "ping on every check".
- This keeps `check` actionable without being noisy during normal polling.

## Scenario
1. Agent A claims `src/app.txt`.
2. Agent B runs `check --path src/app.txt` and sees `warn` with an action plan that includes pinging `@A ... Are you working on it?`.
3. Agent B posts that ping message to A.
4. Agent B runs `check --path src/app.txt` again while A's claim is still active.
5. `check` remains `warn`, still includes `read` and `retry` steps, but does not repeat the same blocker-ping suggestion.

