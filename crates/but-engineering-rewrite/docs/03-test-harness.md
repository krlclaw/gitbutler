# Test Harness

The rewrite spec (`docs/02-spec.md`) requires a disposable-repo harness that can simulate two agents (A/B) and exercise conflict, TTL expiry, and habit-formation behaviors.

## Location

- Harness: `crates/but-engineering-rewrite/harness/`
- Cases (spec as markdown): `crates/but-engineering-rewrite/harness/cases/`

## How To Run

From the repo root:

```bash
./crates/but-engineering-rewrite/harness/run.sh
```

What it does:

- Uses a stub executable if Rust tooling is unavailable, or builds the crate binary if needed.
- For each case, creates a temporary git repo (committed `src/app.txt`).
- Invokes the CLI twice to simulate agents via `--agent-id A` / `--agent-id B`.

## Current Status

The harness is expected to fail right now because the binary is a stub that prints `NOT_IMPLEMENTED`.

To point the harness at a different binary (e.g. when iterating), set:

```bash
BIN=/path/to/but-engineering-rewrite ./crates/but-engineering-rewrite/harness/run.sh
```
