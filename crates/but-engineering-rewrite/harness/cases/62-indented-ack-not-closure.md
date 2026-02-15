# Case 62: Indented Ack Is Not Closure (Still Needs Ack)

Humans often paste prior context as an indented block (Markdown "indented code block" style).

If an unread update contains an indented `@B: ack: ...` snippet as prior context, it should **not**
be treated as explicit closure directed at `B`. The real update still needs a closed-loop ack, so
`check --path` should suggest `@A: ack: ...` for the unread author.

