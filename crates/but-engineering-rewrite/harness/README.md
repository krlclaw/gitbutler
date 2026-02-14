# Test Harness (Disposable Repo)

This harness creates temporary git repos and simulates two agents (A/B) by
invoking the `but-engineering-rewrite` CLI.

It is expected to **fail** until the real CLI behaviors are implemented.

If `cargo` is unavailable, the harness falls back to a stub executable at
`crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

Run:

```bash
./crates/but-engineering-rewrite/harness/run.sh
```
