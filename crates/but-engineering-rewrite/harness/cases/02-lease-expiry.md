# Case 02: Lease Expiry (TTL)

Goal: claims are leases; after TTL expiry, other agents can proceed.

## Setup

- Create a fresh temp git repo with a committed file `src/app.txt`.

## Actions (expected future CLI)

Agent A:

```bash
but-engineering-rewrite --agent-id A claim --path src/app.txt --ttl 2s
```

Wait for expiry:

```bash
sleep 3
```

Agent B:

```bash
but-engineering-rewrite --agent-id B claim --path src/app.txt --ttl 15m
but-engineering-rewrite --agent-id B check --path src/app.txt
```

## Expected

- B can acquire the claim after TTL expiry.
- B `check` returns `decision: allow` with `reason_code: no_conflict`.

## Notes

This case should fail until:

- TTL is enforced
- expired claims are ignored/cleaned up deterministically

