# Case 11: Discovery Provenance (agent_id in brief)

## Why
High-signal discoveries only help a team if other agents can follow up with the right person. Any surfaced discovery should preserve provenance (`agent_id`) so coordination can be routed correctly.

## Scenario
1. Agent A posts a high-signal discovery payload.
2. Agent B runs `brief --type discovery`.

## Expected
- The surfaced discovery in `brief` includes `"agent_id":"A"`.
- `next_steps` still derives an actionable command from the discovery payload.

