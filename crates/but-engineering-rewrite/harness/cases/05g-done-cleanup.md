# Case 05g: Done (Cleanup + Completion Summary)

## Goal
When an agent finishes a chunk of work, they need a single command that:
- releases their active leases (so others can proceed),
- clears their `status`/`plan` (so the team view stays current),
- posts a short completion summary into the shared channel.

This is intentionally "one button" behavior to reduce coordination friction.

## Setup
- Agent A:
  - sets `status` and `plan`,
  - claims `src/app.txt`.

## Steps
1) Agent A runs:
   - `done <summary...>`
2) Agent B runs:
   - `check --path src/app.txt`
   - `agents`
   - `read --type message`

## Expected
- `check` returns `allow` / `no_conflict` (A's claim is released).
- `agents` no longer shows A's prior status/plan values.
- `read --type message` includes A's "DONE: <summary>" completion message (with provenance via `agent_id` in the message object).

