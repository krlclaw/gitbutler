# Case 35: Status/Plan TTL Staleness Surfaced

Goal: coordination output should surface when a blocking agent has gone stale (no recent updates), even if their claim TTL is still active. This is a closed-loop behavior: detect staleness, say it explicitly, and provide an actionable next step to request an update.

## Setup

- Fresh temp git repo with committed `src/app.txt`.
- Agent A:
  - sets an initial `status` and `plan`
  - claims `src/app.txt` with TTL `15m`

## Actions (expected CLI)

- Wait long enough to exceed a configurable staleness threshold (harness uses `COORD_STALE_SECONDS=2`).
- Agent B runs:

```bash
COORD_STALE_SECONDS=2 but-engineering-rewrite --agent-id B check --path src/app.txt
```

## Expected

- Output is JSON.
- `check` still shows the claim conflict (warn/deny as usual).
- Output includes a staleness indicator for Agent A (explicitly marked stale).
- Output includes a concrete, actionable `post` command template asking A to update `status` and `plan`.

