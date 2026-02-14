# Case 05: Dependency Hint (Provider/Consumer API)

Goal: avoid hard locks, but still help agents notice when they should coordinate because one agent is changing an API surface that another agent intends to consume.

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.

## Actions (expected future CLI)

Agent A declares they are working on an API surface (provider side):

```bash
but-engineering-rewrite --agent-id A post --type declaration --json \
  '{"scope":"component:sync","tags":["component/api","provider"],"surface":["SyncService::push","SyncService::pull"],"note":"Refactor request/response types"}'
```

Agent B declares intent to consume that API surface:

```bash
but-engineering-rewrite --agent-id B post --type intent --json \
  '{"scope":"component:sync","tags":["consumer"],"surface":["SyncService::push"],"note":"Implement CLI command that calls push"}'
```

Agent B checks before editing (danger point):

```bash
but-engineering-rewrite --agent-id B check --path src/app.txt
```

## Expected

- Output is JSON.
- `check` still returns `allow` (no hard locks).
- Output includes `dependency_hints` when B's intent overlaps A's declared API surface.
- The hint is explicit about:
  - `why` (what overlapped)
  - `next_step` (how to coordinate)

## Notes

Heuristic can be simple: intersect intent `surface[]` with any other agent's declared `surface[]` when declaration tags include `api`.
