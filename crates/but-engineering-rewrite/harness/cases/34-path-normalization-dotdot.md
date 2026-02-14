# Case 34: Path Normalization (.. Segments)

## Goal
Avoid false-positive claim overlaps when paths include `..` segments.

Coordination wrappers sometimes construct relative paths like `src/../notes.txt`.
If we treat overlap purely as a string prefix, `src/../notes.txt` incorrectly looks
like it is under the claimed `src/` directory, causing needless "claimed_by_other"
warnings and coordination churn.

## Harness Steps
1. Agent A: `claim --path src/ --ttl 15m`
2. Agent B: `check --path src/../notes.txt`

## Expected
- Decision is `allow`.
- `reason_code` is `no_conflict` (not `claimed_by_other`).

