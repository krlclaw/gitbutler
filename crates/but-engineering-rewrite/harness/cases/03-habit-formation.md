# Case 03: Habit Formation (Prompt Nudge)

Goal: `eval user-prompt-submit` stays short but reflects live state (agents/claims/unread).

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.
- Agent A posts intent and claims a file.

## Actions (expected future CLI)

Baseline (no state):

```bash
but-engineering-rewrite --agent-id A eval user-prompt-submit
```

Create state:

```bash
but-engineering-rewrite --agent-id A post "I will edit src/app.txt"
but-engineering-rewrite --agent-id A claim --path src/app.txt --ttl 15m
```

Observe updated nudge:

```bash
but-engineering-rewrite --agent-id B eval user-prompt-submit
```

## Expected

- Output is plain text (not JSON), suitable for a hook.
- Contains a stable short 1-liner reminder (from `docs/02-spec.md`).
- Includes novel state that changes after A posts/claims:
  - active agents summary
  - recent/unread messages preview
  - current claims summary

## Notes

This case should fail until:

- a channel exists
- sessions/agents can be derived or approximated
- claims are visible in the prompt output

