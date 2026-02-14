# Case 07: Release Claim (Explicit Unclaim)

## Why this case exists
Leases solve the “forgetting to announce/claim” problem, but TTL alone is not a great UX:

- An agent can finish quickly and wants to stop blocking/warning others immediately.
- Long TTLs are useful for safety, but they should be releasable when the work is done.

This case adds a minimal escape hatch: `release --path <path>`.

## Scenario
1. Agent A claims `src/app.txt`.
2. Agent B checks that path and gets a warning (`warn` + `claimed_by_other`).
3. Agent A explicitly releases the claim for that path.
4. Agent B checks again and gets `allow` + `no_conflict`.

## CLI Surface
- `but-engineering-rewrite --agent-id A release --path src/app.txt`

## Notes
- This is intentionally narrow: release only affects the caller's claim for that path.
- This does not introduce hard locks; it simply deletes (or marks inactive) the lease row.

