# Case 17: Brief `--all` (Escape Hatch For Low-Signal Discoveries)

Goal: low-signal discoveries should not clutter the default `brief`/`digest`, but agents still need an escape hatch to inspect everything when debugging or coordinating. `brief --all` should include both low- and high-signal discoveries and derive `next_steps` from the same set.

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.

## Actions (expected future CLI)

Agent A posts a low-signal discovery:

```bash
but-engineering-rewrite --agent-id A post --type discovery --json \
  '{"signal":"low","title":"ERW Case 17: low signal","evidence":[{"kind":"file","path":"src/app.txt","note":"still should be inspectable"}],"suggested_action":{"kind":"run","cmd":"echo ERW_CASE17_LOW_UNIQUE","note":"should only surface with --all"}}'
```

Agent A posts a high-signal discovery:

```bash
but-engineering-rewrite --agent-id A post --type discovery --json \
  '{"signal":"high","title":"ERW Case 17: high signal","evidence":[{"kind":"cmd","cmd":"git status --porcelain","note":"prove next_steps still works"}],"suggested_action":{"kind":"run","cmd":"echo ERW_CASE17_HIGH_UNIQUE","note":"still visible without --all"}}'
```

Agent B reads discoveries (default brief, high-signal only):

```bash
but-engineering-rewrite --agent-id B brief --type discovery
```

Agent B reads discoveries (escape hatch, include all):

```bash
but-engineering-rewrite --agent-id B brief --type discovery --all
```

## Expected

- Output is JSON.
- Default `brief` includes the high-signal discovery and excludes the low-signal one.
- `brief --all` includes both.
- `next_steps` in `brief --all` includes commands derived from both discoveries.

