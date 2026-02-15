# Case 54: Tasklist Ack Is Closure (No Ack Ping-Pong)

People often respond using GitHub-style task lists:

```text
- [x] @B: ack: saw it
```

That should still count as explicit closure directed at `B`, so `check --path` must **not**
suggest an `@A: ack:` back (avoids ack ping-pong noise).

