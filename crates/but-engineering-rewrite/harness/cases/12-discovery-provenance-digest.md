# Case 12: Discovery Provenance (agent_id in digest)

## Goal
When an agent compacts discoveries into a `digest`, the output must still preserve **provenance** so a consumer can follow up with the correct agent.

## Setup
- Agent A posts a high-signal `discovery` (structured JSON) with evidence + a runnable suggested action.
- Agent B requests `digest --type discovery`.

## Expected behavior
- `digest` returns a compact `discoveries[]` list that includes at least `title` and `agent_id`.
- `next_steps[]` is still derived from the posted discovery payload (`suggested_action.cmd`).

## Why this matters
Without provenance, the coordination substrate forces humans/agents to guess who to contact. Digest should be concise, not anonymous.

