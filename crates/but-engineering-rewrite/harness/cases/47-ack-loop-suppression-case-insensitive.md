# Case 47: Ack Loop Suppression (Case-Insensitive Ack:)

Prevent pointless ack ping-pong even when humans vary capitalization.

The `check --path` action plan should not suggest acknowledging an unread update that is itself an acknowledgement directed at the requester, even if the author wrote `Ack:` instead of `ack:`.

## Scenario

1. Agent A posts a coordination note mentioning `src/app.txt`.
2. Agent B runs `check --path src/app.txt` and is prompted to explicitly ack A.
3. B posts an ack variant: `@A: Ack: ... src/app.txt ...`.
4. Agent A runs `check --path src/app.txt` and sees B's ack as an unread relevant update.
5. `check` must **not** suggest an `@B: ack: ...` step in response to that ack variant.

