# Case 29: Action Plan Releases Most Specific Claim

## Goal
When `check` is blocked and a blocking agent holds **multiple overlapping claims**
(e.g. both `src/` and `src/app.txt`), the per-agent action plan should recommend
releasing the **most specific** overlap first.

This keeps coordination low-churn: releasing a narrow file claim is less
disruptive than releasing a broad directory claim.

## Harness Steps
1. Agent A: `claim --path src/ --ttl 15m`
2. Agent A: `claim --path src/app.txt --ttl 15m`
3. Agent B: `check --path src/app.txt`

## Expected
- Decision is `warn`.
- `action_plan_by_agent["A"]` includes `release --path src/app.txt`.
- `action_plan_by_agent["A"]` does **not** include `release --path src`.

