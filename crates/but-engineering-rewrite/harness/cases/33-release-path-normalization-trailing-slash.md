# Case 33: Release Path Normalization (Trailing /)

Goal: prevent stale coordination leases when wrappers or humans mix `src` vs `src/` spellings. Releasing a directory claim should work even if the release path has a trailing slash.

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.

## Actions (expected future CLI)

Agent A:

```bash
but-engineering-rewrite --agent-id A claim --path src --ttl 15m
but-engineering-rewrite --agent-id A release --path src/
```

Agent B:

```bash
but-engineering-rewrite --agent-id B check --path src/app.txt
```

## Expected

- Agent A `release` returns `{"ok":true}`.
- Agent B `check` returns JSON:
  - `decision`: `allow`
  - `reason_code`: `no_conflict`

