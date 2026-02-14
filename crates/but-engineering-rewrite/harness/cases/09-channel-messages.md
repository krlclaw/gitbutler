# Case 09: Channel Messages (post/read transcript)

Goal: coordination requires a shared transcript. Agents must be able to post plain text messages and read them back with provenance.

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.

## Actions (expected future CLI)

Agent A posts a plain message:

```bash
but-engineering-rewrite --agent-id A post "@B: I'm going to refactor src/app.txt next"
```

Agent B posts a reply:

```bash
but-engineering-rewrite --agent-id B post "@A: ack, I'm only reading for now"
```

Agent B reads the transcript of plain messages:

```bash
but-engineering-rewrite --agent-id B read --type message
```

## Expected

- Output is JSON.
- `read --type message` returns `{"ok":true,"kind":"message","messages":[...]}`.
- Each message includes provenance (`agent_id`) and the original `text` content.

