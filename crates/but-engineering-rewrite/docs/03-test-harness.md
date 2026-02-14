# Test Harness

The rewrite spec (`docs/02-spec.md`) requires a disposable-repo harness that can simulate two agents (A/B) and exercise conflict, TTL expiry, and habit-formation behaviors.

## Eval Vocabulary

- **Task**: A scenario we care about (e.g. "two-agent collision warning is emitted").
- **Trial**: One execution of the full case suite (or subset) against a fresh set of disposable repos. Trials are used to measure flakiness.
- **Grader**: The harness logic that decides pass/fail for a case by asserting on CLI outputs.
- **Trace**: A structured log of every CLI invocation the harness performed (command, args, cwd/repo, stdout/stderr, exit code).
- **Outcome**: The per-case result (pass/fail) within a trial; aggregated into a per-trial result.

## Location

- Harness: `crates/but-engineering-rewrite/harness/`
- Cases (spec as markdown): `crates/but-engineering-rewrite/harness/cases/`

## How To Run

From the repo root:

```bash
./crates/but-engineering-rewrite/harness/run.sh
```

### Traces

Each harness run writes a per-run output folder and a JSONL trace at:

- `crates/but-engineering-rewrite/harness/out/<timestamp>/trace.jsonl`

Each JSONL row is one CLI invocation with fields like `trial`, `case`, `agent_id`, `cwd`/`repo`, `cmd`, `args`, `stdout`, `stderr`, and `exit_code`.

### Trials

Run the suite multiple times (default is 1 trial):

```bash
HARNESS_TRIALS=3 ./crates/but-engineering-rewrite/harness/run.sh
```

The harness prints `Trial i/N: PASS|FAIL` and exits non-zero if any trial fails.

### Replay (Skeleton)

To compare a new run against a saved trace:

```bash
HARNESS_REPLAY_DIR=crates/but-engineering-rewrite/harness/out/<timestamp> \
  ./crates/but-engineering-rewrite/harness/run.sh
```

Replay currently checks:

- per-invocation `exit_code` matches the saved trace
- if the saved trace's `stdout` is JSON, the new `stdout` must be JSON and contain the same top-level keys

What it does:

- Uses a stub executable if Rust tooling is unavailable, or builds the crate binary if needed.
- For each case, creates a temporary git repo (committed `src/app.txt`).
- Invokes the CLI twice to simulate agents via `--agent-id A` / `--agent-id B`.

## Current Status

The harness is expected to pass when run against either:
- the Rust binary (built via Cargo), or
- the bundled stub (for environments without Rust tooling).

To point the harness at a different binary (e.g. when iterating), set:

```bash
BIN=/path/to/but-engineering-rewrite ./crates/but-engineering-rewrite/harness/run.sh
```
