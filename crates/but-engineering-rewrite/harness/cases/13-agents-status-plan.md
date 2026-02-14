# Case 13: Agents + Status/Plan (Visibility)

## Why

Coordination tools need lightweight "who is doing what" visibility so agents can
avoid stepping on each other without hard locks. The spec includes:

- `status` / `plan` for short self-reported state
- `agents` to list active agents and their current state

## Harness Steps

1) Agent A sets a status and plan.
2) Agent B runs `agents` and sees A's `status` and `plan`.
3) Agent B also sets its own status so `agents` returns multiple agents.

## Expected Output (minimal)

- `status ...` returns JSON containing `"ok":true`
- `plan ...` returns JSON containing `"ok":true`
- `agents` returns JSON containing:
  - `"ok":true`
  - `"agents"` array with entries including `agent_id`, `status`, `plan`

