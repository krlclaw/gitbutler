# Requirements (living)

This doc captures Kiril’s product requirements/preferences for the agent coordination tool.

## Goal
Build a **valuable agent coordination substrate** for agents operating in the **same worktree** (initially). It should also help agents on different tasks (human team-like).

## Functional priorities
1) **Collision detection without hard locks** (advisory by default; keep things fast). Provide strict/deny only as an escape hatch.
2) **High-signal discoveries**: share only truly valuable findings, structured with evidence + suggested action.
3) **Dependency hints**: help an agent recognize when its work depends/should depend on another agent’s work.

## Design preferences
- **Agent-agnostic**: should work for Claude Code, Codex, OpenCode, Cursor, etc.
- Prefer to work **without hooks**; hooks only as optional QoL adapters.
- Don’t be constrained by a Slack/chat metaphor; use structured events and an actionable digest/brief.
- Future direction: could become a **distributed system** coordinating agents across machines.

## Process requirements
- Tests/harness are critical; progress means scenarios passing.
- Keep code minimal; declare dead ends and restart if needed.
- Record key requirements + research as markdown artifacts that agents can read.
