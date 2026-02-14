# Case 23: Path Normalization (Leading ./)

## Why
In a real repo, agents will often refer to the same file with slightly different
spellings (for example `src/app.txt` vs `./src/app.txt`). Coordination becomes
unreliable if these are treated as distinct paths.

This case asserts that:
- A claim on `src/app.txt` conflicts with a `check` on `./src/app.txt`.

## Expected Behavior
- Default mode remains advisory: `check` returns `warn` + `claimed_by_other`.

## Minimal CLI Surface
- `claim --path <path> --ttl <duration>`
- `check --path <path>`

