# Case 01: Two-Agent Collision (Advisory By Default)

Goal: prove the core property: a file with an active intent by agent A is detected by agent B at a danger point (`check`) without hard locking by default.

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.

## Actions (expected future CLI)

Agent A:

```bash
but-engineering-rewrite --agent-id A claim --path src/app.txt --ttl 15m
```

Agent B:

```bash
but-engineering-rewrite --agent-id B check --path src/app.txt
```

## Expected

Agent B `check` returns JSON:

- `decision`: `warn`
- `reason_code`: `claimed_by_other`
- `blocking_agents`: includes `A`
- `action_plan`: includes steps like `read_channel`, `post_intent`, `wait_for_release`, `retry`

## Notes

This case should fail until:

- `claim` persists a lease for `src/app.txt`
- `check` consults active leases and emits the contract from `docs/02-spec.md`
