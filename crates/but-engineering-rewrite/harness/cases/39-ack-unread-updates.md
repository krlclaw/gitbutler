# Case 39: Ack Unread Updates (Closure + Anti-Spam)

## Goal
When an agent posts a relevant coordination update (without taking a claim), `check --path` should recommend a concrete acknowledgement step so the other agent knows the message was seen.

## Why This Matters
- Coordination failures often happen when updates are broadcast but never explicitly acknowledged.
- A closed loop (post -> surfaced -> ack) reduces duplicate work and reduces "did you see this?" follow-ups.
- The suggestion must be low-noise: the same ack should not be repeated once the update has been surfaced.

## Scenario
1. Agent A posts a plain-text message mentioning `src/app.txt`.
2. Agent B runs `check --path src/app.txt`.
3. `check` returns `allow`, includes the unread update payload, and includes an action plan step that suggests posting an explicit `@A: ack: ...` message.
4. A second `check --path src/app.txt` by B does not repeat the same update nor the ack suggestion (cursor advanced).

