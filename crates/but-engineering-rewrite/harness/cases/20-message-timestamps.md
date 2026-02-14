# Case 20: Message Timestamps (Transcript Ordering)

Goal: coordination requires a shared transcript that can be presented in a stable order and annotated with timing. `read` should include a timestamp for each message so UIs can render "when" and sort deterministically.

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.

## Actions (expected future CLI)

Agent A posts a message, then Agent B posts a reply:

```bash
but-engineering-rewrite --agent-id A post "starting work on src/app.txt"
but-engineering-rewrite --agent-id B post "ack, I'm just reviewing"
```

Agent C reads the default transcript:

```bash
but-engineering-rewrite --agent-id C read
```

## Expected

- Output is JSON.
- Default `read` returns `{"ok":true,"kind":"message","messages":[...]}`.
- Each message includes:
  - `agent_id`
  - `text`
  - `created_at_ms` (integer unix millis)
- `messages[]` are returned in non-decreasing `created_at_ms` order.

