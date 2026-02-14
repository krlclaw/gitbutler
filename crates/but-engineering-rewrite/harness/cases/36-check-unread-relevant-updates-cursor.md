# Case 36: Check Surfaces Unread Relevant Updates (Cursor Advances)

Goal: `check --path` should surface unread relevant coordination updates since the caller last checked that path, and then advance a cursor so the same updates are not repeated on subsequent checks.

## Setup

- Fresh temp git repo with committed `src/app.txt`.
- Agent A posts a channel message that is relevant to `src/app.txt` (includes the path).

## Actions (expected CLI)

Agent B runs `check --path src/app.txt` twice:

```bash
but-engineering-rewrite --agent-id B check --path src/app.txt
but-engineering-rewrite --agent-id B check --path src/app.txt
```

## Expected

- Output is JSON.
- First check includes an "unread relevant updates since last seen" surface that includes Agent A's message.
- Second check does not repeat the same message (cursor advanced).

