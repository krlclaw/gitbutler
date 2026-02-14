# Case 16: Read Default (Channel Transcript)

Goal: The most common "read" operation in a coordination tool is to see the shared channel transcript. `read` with no flags should default to showing `message` entries so wrappers (and humans) don't need to remember `read --type message`.

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.

## Actions (expected future CLI)

Agent A posts a plain text message:

```bash
but-engineering-rewrite --agent-id A post "I'm starting work on src/app.txt"
```

Agent B runs `read` with no `--type`:

```bash
but-engineering-rewrite --agent-id B read
```

## Expected

- Output is JSON.
- Output includes `{"kind":"message","messages":[...]}`.
- Messages include `agent_id` provenance and the posted text.

