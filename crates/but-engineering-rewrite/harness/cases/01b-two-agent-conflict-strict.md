# Case 01b: Two-Agent Collision (Strict Deny Mode)

Goal: prove the escape hatch: `check --strict` can deny when another agent has an active intent on the file.

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.

## Actions (expected future CLI)

Agent A:

```bash
but-engineering-rewrite --agent-id A claim --path src/app.txt --ttl 15m
```

Agent B (strict):

```bash
but-engineering-rewrite --agent-id B check --path src/app.txt --strict
```

## Expected

Agent B `check` returns JSON:

- `decision`: `deny`
- `reason_code`: `claimed_by_other`
- `blocking_agents`: includes `A`
- `action_plan`: includes steps like `read_channel`, `post_intent`, `wait_for_release`, `retry`

