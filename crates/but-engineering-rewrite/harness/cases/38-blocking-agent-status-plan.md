# Case 38: Check Includes Blocking Agent Status/Plan Snapshot

Goal: when `check --path` is blocked by another agent's claim, the output should include that blocking agent's current `status` and `plan` (if set). This reduces coordination round-trips because the caller can immediately see "what they're doing" and "what's next" without first running a separate `agents` query.

## Setup

- Fresh temp git repo with committed `src/app.txt`.
- Agent A:
  - sets `status` and `plan`
  - claims `src/app.txt` with TTL `15m`

## Actions (expected CLI)

- Agent B runs:

```bash
but-engineering-rewrite --agent-id B check --path src/app.txt
```

## Expected

- Output is JSON.
- `decision` is `warn` (or `deny` under `--strict`) as usual.
- Output includes a `blocking_agents_state` field containing an entry for Agent A with:
  - `agent_id`
  - `status`
  - `plan`
  - `updated_at_ms`

