Avoid false closure: users paste prior context in fenced code blocks, which can include strings like `@<me>: resolve: ...`.

`check --path` must not treat a `resolve:` snippet inside a code block as explicit closure directed at me; it is just quoted context, and I should still be prompted to acknowledge the real update.

Scenario:
1. Agent A posts an update mentioning `src/app.txt` and includes a fenced code block containing a line like `@B: resolve: go ahead`.
2. Agent B runs `check --path src/app.txt`.

Expected:
- The update surfaces under `unread_relevant_updates`.
- The action plan suggests a single `@A: ack:` (code-block resolve is not closure).

