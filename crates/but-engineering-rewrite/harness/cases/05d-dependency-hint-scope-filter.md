# Case 05d: Dependency Hint (Scope Filter)

## Goal
Avoid noisy dependency hints when different components reuse the same surface token name.

## Setup
- Agent A posts a `declaration` tagged as API-ish (`tags[]` contains `"api"`), but with scope `component:auth` and a surface token `SyncService::push`.
- Agent B posts an `intent` with scope `component:sync` and surface token `SyncService::push`.

## Expected Behavior
- `check` remains `"decision":"allow"` (no hard locks).
- `dependency_hints` is empty because the scopes differ (`component:auth` vs `component:sync`), even though the surface token matches.

