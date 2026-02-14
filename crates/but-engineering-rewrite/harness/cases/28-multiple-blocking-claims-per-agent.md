# Case 28: Multiple Blocking Claims Per Agent

## Goal
When an agent holds multiple overlapping claims (e.g. a directory claim plus a file claim),
`check` should report **all** overlapping `blocking_claims` for that agent, not just a
single "best" path. This makes conflicts more actionable (callers can see what would
need to be released/updated).

## Harness Steps
1. A: `claim --path src/ --ttl 15m`
2. A: `claim --path src/app.txt --ttl 15m`
3. B: `check --path src/app.txt`

## Expected
- Decision is `warn` (blocked by A).
- `blocking_agents` includes `A`.
- `blocking_claims` contains **both** `src` and `src/app.txt` for agent `A`.

