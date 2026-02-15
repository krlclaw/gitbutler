# Case 64: Bolded Resolve Is Closure (Markdown Emphasis)

Humans sometimes format explicit closure directives as a "reply checklist" line like
`- **@<me>: resolve:** ...` for readability.
That is still directed closure and should not trigger an extra `@X: ack:` back (avoid ack ping-pong noise).

## Scenario

1. Agent A holds a broad claim that blocks B.
2. B asks for clarification.
3. A replies with a checklist entry: `- **@B: resolve:** ...` and releases.
4. B runs `check --path`.
5. `check` surfaces the unread repair update once, but does **not** suggest acknowledging it back.

