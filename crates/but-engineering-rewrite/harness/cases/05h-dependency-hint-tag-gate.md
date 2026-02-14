# Case 05h: Dependency Hint (API Tag Gate)

## Why
Dependency hints should be high-signal. Using a naive substring match for “api”
in tags creates false positives (e.g. a tag like `capistrano` contains `api`).

## Harness Steps
1. Agent A posts a `declaration` with tags that include a non-API tag containing
   the substring `api` (e.g. `capistrano`) and a surface token `SyncService::push`.
2. Agent B posts an `intent` with the same scope and overlapping surface token.
3. Agent B runs `check`.

## Harness Assertions
- `check` returns `"decision":"allow"`.
- `dependency_hints` is an empty array (no false positive hint).

