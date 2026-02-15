# Case 63: Comma Ack Is Closure (No Ack Ping-Pong)

## Goal
Treat a common punctuation variant of explicit closure as real closure:
`@<me>: ack, ...` should behave like `@<me>: ack: ...` so `check --path` does not suggest an
`@X: ack:` back (avoids ack ping-pong noise).

## Why This Matters
- Humans often write “ack, thanks” (comma) instead of the stricter `ack:` format.
- If we fail to treat this as closure, wrappers keep nagging for an acknowledgement that already happened.
- The useful behavior is: surface the message once, then let the requester proceed without extra back-and-forth.

## Scenario
1. Agent A posts a relevant update that contains a line starting with `@B: ack, ...` and mentions `src/app.txt`.
2. Agent B runs `check --path src/app.txt` and sees the message as an unread relevant update.
3. `check` does **not** suggest `@A: ack:` back, since the update itself explicitly acks B.
