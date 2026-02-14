# Case 05b: Check Action Plan (Includes Blocking Agent)

Goal: `check` responses should be machine-actionable, and when another agent is blocking (via an active claim) the output should name the blocking agent and include a concrete next step that pings them.

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.

## Actions (expected future CLI)

Agent A claims the file:

```bash
but-engineering-rewrite --agent-id A claim --path src/app.txt --ttl 15m
```

Agent B checks before editing:

```bash
but-engineering-rewrite --agent-id B check --path src/app.txt
```

## Expected

- Output is JSON.
- `decision` is `warn` (advisory-by-default).
- Output includes:
  - `blocking_agents` containing `A`
  - `action_plan` containing a command that references `@A` (explicit coordination step)

