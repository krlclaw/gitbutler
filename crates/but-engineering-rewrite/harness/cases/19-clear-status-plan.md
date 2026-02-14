# Case 19: Clear Status/Plan (Avoid Stale Coordination)

Goal: status/plan are useful only if they can be explicitly cleared. Stale status is worse than missing status because it causes false assumptions and pings.

## Setup

- Create a fresh temp git repo.

## Actions (expected future CLI)

Agent A sets both fields:

```bash
but-engineering-rewrite --agent-id A status "Working on API refactor"
but-engineering-rewrite --agent-id A plan "1) change types 2) update callers 3) run tests"
```

Agent A clears only status (plan remains):

```bash
but-engineering-rewrite --agent-id A status --clear
```

Agent B lists agents:

```bash
but-engineering-rewrite --agent-id B agents
```

Agent A clears plan too:

```bash
but-engineering-rewrite --agent-id A plan --clear
```

## Expected

- `status --clear` sets status to null without clearing plan.
- `plan --clear` sets plan to null.
- `agents` reflects the cleared fields so coordination views don't retain stale signals.

