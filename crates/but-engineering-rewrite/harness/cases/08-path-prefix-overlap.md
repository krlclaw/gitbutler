# Case 08: Path Prefix Overlap (Directory Claim)

## Why
Agent claims are often coarse (a folder or component), while edits are fine-grained
(a single file). We still want collision detection to be useful without requiring
agents to claim every individual file.

This case asserts that the coordinator treats ancestor/descendant paths as
overlapping:
- Claiming `src/` should conflict with another agent checking `src/app.txt`.

## Expected Behavior
- Default mode remains advisory: `check` returns `warn` + `claimed_by_other`.
- (Strict behavior is covered elsewhere; this case focuses on path overlap.)

## Minimal CLI Surface
- `claim --path <path> --ttl <duration>`
- `check --path <path>`

