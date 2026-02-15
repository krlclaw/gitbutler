# Case 69: Inline-Code Resolve Is Closure (No Ack Needed)

Humans often format directive-like tokens using inline code in chat or PR comments, e.g.:

- `@B: resolve:` not touching `src/app.txt`; releasing claim now.

This should still count as explicit closure directed at the requester, so `check --path`:

- surfaces the unread update, and
- **does not** suggest an `@A: ack:` back (avoid redundant ack ping-pong).

