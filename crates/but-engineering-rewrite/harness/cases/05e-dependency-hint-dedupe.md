# Case 05e: Dependency Hint (Dedupe Per Provider)

## Goal
Avoid noisy dependency hints when the same provider posts multiple declarations for the same scope.

This is realistic in a live system because declarations may be reposted as the agent iterates or "renews" its status.

## Setup
- Agent A posts two `declaration` payloads for the same `scope` with overlapping surface tokens (simulating repeat/renew).
- Agent B posts an `intent` that overlaps.

## Expected Behavior
- `check` remains `"decision":"allow"` (no hard locks).
- `dependency_hints` contains a hint for provider `A`.
- The hints are deduped so provider `A` appears only once for that scope (keep the newest / best hint).

