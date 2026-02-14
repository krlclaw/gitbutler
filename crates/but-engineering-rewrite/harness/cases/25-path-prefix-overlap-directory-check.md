# Case 25: Path Prefix Overlap (Directory Check)

## Why
Agents often scope work by directory (`src/`) instead of a single file. Coordination is only useful if
checking a directory detects conflicts with file-level claims inside that directory, and includes the
specific blocking claim path(s) so the agent can resolve the conflict quickly.

## Script
```bash
but-engineering-rewrite --agent-id A claim --path src/app.txt --ttl 15m
but-engineering-rewrite --agent-id B check --path src/
```

## Expected
- Decision is `warn` (advisory by default).
- Output includes `blocking_claims` with `{"agent_id":"A","path":"src/app.txt",...}`.

