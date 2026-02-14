# Case 04: High-Signal Discovery Propagation (Brief + Digest)

Goal: agent A can post discoveries, but only truly high-signal ones propagate by default. Agent B can fetch them via `brief` (full payload) or `digest` (summary) and get actionable next steps derived from the discovery payload.

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.

## Actions (expected future CLI)

Agent A posts a low-signal discovery (should not propagate by default):

```bash
but-engineering-rewrite --agent-id A post --type discovery --json \
  '{"signal":"low","title":"Minor note: update comment wording","evidence":[{"kind":"file","path":"src/app.txt","note":"comment style nit"}],"suggested_action":{"kind":"run","cmd":"echo ERW_CASE04_LOW_UNIQUE","note":"low-signal: should not surface"}}'
```

Agent A posts a high-signal discovery (should propagate):

```bash
but-engineering-rewrite --agent-id A post --type discovery --json \
  '{"signal":"high","title":"Lockfile indicates workspace mismatch","evidence":[{"kind":"file","path":"Cargo.lock","note":"has deps not in Cargo.toml"},{"kind":"cmd","cmd":"git status --porcelain","note":"repo is clean; mismatch is from lockfile"}],"suggested_action":{"kind":"run","cmd":"cargo update -p <crate>","note":"align lockfile"}}'
```

Agent B reads discoveries (brief):

```bash
but-engineering-rewrite --agent-id B brief --type discovery
```

Agent B reads discoveries (digest):

```bash
but-engineering-rewrite --agent-id B digest --type discovery
```

## Expected

- Output is JSON.
- By default, `brief`/`digest` include only high-signal discoveries.
- `brief` includes the full discovery payload (title/evidence/suggested_action).
- `digest` includes a smaller summary view of discoveries.
- Both include actionable `next_steps` (commands) derived from the discovery payload.

## Notes

This case is intentionally minimal: we only need enough to make the harness pass and to validate the “structured discovery” concept.
