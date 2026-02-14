# Case 10: Claims Listing (Visibility)

Goal: coordination requires visibility. An agent should be able to list active claims (leases) to see who is working on what, without having to guess paths or trigger a conflicting `check`.

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.

## Actions (expected future CLI)

Agent A claims a path:

```bash
but-engineering-rewrite --agent-id A claim --path src/app.txt --ttl 15m
```

Agent B lists active claims:

```bash
but-engineering-rewrite --agent-id B claims
```

## Expected

- Output is JSON.
- `claims` returns `{"ok":true,"claims":[...]}`.
- Each claim includes `path`, `agent_id`, and an expiry timestamp.

