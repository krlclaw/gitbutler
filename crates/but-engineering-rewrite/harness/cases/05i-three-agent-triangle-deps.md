# Case 05i: 3-Agent Triangle + Dependency Hints (Noise Control)

## Goal
Exercise a realistic 3-agent situation in one scenario:

- **Triangle claim conflict**: A/B/C all hold overlapping claims so each agent's `check` hits a conflict.
- **Advisory vs strict**: `check` warns by default, but denies with `--strict`.
- **Dependency chain**: B's intent overlaps A's API declaration and should emit a dependency hint.
- **Scope/noise control**: C uses the same surface token name in a different scope and must not create (or receive) a dependency hint.
- **Dedupe**: A may post multiple declarations for the same scope; B should see a single hint for provider A.

## Notes
This case intentionally mixes **claim conflicts** (coordination locks) with **dependency hints** (API producer/consumer coordination).

