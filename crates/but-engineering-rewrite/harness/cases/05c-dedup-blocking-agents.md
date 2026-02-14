# Case 05c: Dedupe Blocking Agents (No Duplicate @Agent IDs)

## Scenario
An agent may accidentally (or intentionally) claim the same path multiple times (e.g. renewal heartbeats, retries, wrapper bugs).

When another agent runs `check`, the output should stay **machine-actionable** and **non-noisy**:
- `blocking_agents` must list each blocking agent **at most once**.

## Harness Steps
1. Agent A: `claim --path src/app.txt --ttl 15m`
2. Agent A: `claim --path src/app.txt --ttl 15m` (repeat)
3. Agent B: `check --path src/app.txt`

## Expected Output (B)
- `decision` is `warn` (advisory by default)
- `blocking_agents` is exactly `["A"]` (not `["A","A"]`)

