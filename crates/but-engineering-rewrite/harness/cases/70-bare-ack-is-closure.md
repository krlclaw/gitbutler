# Case 70: Bare Ack Is Closure (No Ack Ping-Pong)

## Goal
Treat a minimal explicit-closure variant as real closure:
`@<me>: ack` (no trailing `:` or punctuation) should behave like `@<me>: ack: ...` so `check --path`
does not suggest an `@X: ack:` back.

## Why This Matters
- Humans often type a short `@B: ack` as a quick confirmation, especially in chat-style threads.
- If we fail to treat this as closure, wrappers create noisy ack ping-pong and reduce signal in the action plan.
- The useful behavior is: surface the message once, then let the requester proceed without extra back-and-forth.

## Scenario
1. Agent A posts a relevant update that starts with `@B: ack` and mentions `src/app.txt`.
2. Agent B runs `check --path src/app.txt` and sees the message as an unread relevant update.
3. `check` does **not** suggest `@A: ack:` back, since the update itself explicitly acks B.

