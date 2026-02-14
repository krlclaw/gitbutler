# Case 06: Inspect Surfaces (Read Declarations + Intents)

Goal: once structured `intent` and `declaration` messages exist (to drive dependency hints), agents should be able to inspect what is currently "in play" without reverse-engineering it from `check` output.

This case ensures the CLI supports a minimal `read --type intent|declaration` for debugging/coordination.

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.

## Actions (expected future CLI)

Agent A posts a declaration (provider-side):

```bash
but-engineering-rewrite --agent-id A post --type declaration --json \
  '{"scope":"component:sync","tags":["component/api","provider"],"surface":["SyncService::push","SyncService::pull"],"note":"Refactor request/response types"}'
```

Agent B posts an intent (consumer-side):

```bash
but-engineering-rewrite --agent-id B post --type intent --json \
  '{"scope":"component:sync","tags":["consumer"],"surface":["SyncService::push"],"note":"Implement CLI command that calls push"}'
```

Agent B reads current declarations:

```bash
but-engineering-rewrite --agent-id B read --type declaration
```

Agent B reads current intents:

```bash
but-engineering-rewrite --agent-id B read --type intent
```

## Expected

- Output is JSON.
- `read --type declaration` returns a `messages` list with at least one entry including:
  - the original payload fields (including `surface`)
  - `agent_id` provenance (`A`)
- `read --type intent` returns a `messages` list with at least one entry including:
  - the original payload fields (including `surface`)
  - `agent_id` provenance (`B`)

