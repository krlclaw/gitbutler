# Case 15: Multi-Blocker Check (Action Plan Mentions Each)

## Scenario
Two different agents have overlapping claims for a file (e.g. one claims the directory
and another claims the specific file). A third agent checks the file.

## Why
Real coordination involves multiple parallel edits. `check` should remain low-noise
and directly actionable:
- `blocking_agents` must include *each* blocking agent once (deduped).
- `action_plan` must include a suggested "ping" step for each blocker (e.g. `@A`, `@C`).

## Harness Assertions
- `check` returns `decision: warn` (advisory, no hard lock).
- Output contains both blocking agent ids.
- Output action plan references both blockers.
