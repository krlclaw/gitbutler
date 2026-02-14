# Design decisions (living)

This file is a running log of architectural/product decisions for the `but-engineering` rewrite.

Format:
- **Decision**
- **Context** (why we had to decide)
- **Options considered**
- **Decision**
- **Consequences / follow-ups**

---

## DD-0001: Keep Git as source of truth; use DB for ephemeral coordination
- **Context:** Coordination needs low-latency shared state (messages, claims) without turning into a distributed system.
- **Options considered:**
  1) Git-only coordination (issues/branches/lock files)
  2) SQLite-in-repo (like current but-engineering)
  3) External service (Redis/etcd/event bus)
- **Decision:** Keep an in-repo SQLite DB as the coordination substrate (messages/claims/status), but treat it as **ephemeral coordination state**, not canonical artifact history.
- **Consequences / follow-ups:**
  - DB path must be deterministic + repo-scoped.
  - Provide `export` tooling later if we want archival.

## DD-0002: Deterministic enforcement only at “danger points”
- **Context:** Purely social protocols are brittle; heavy structural isolation (worktrees everywhere) is expensive.
- **Options considered:**
  1) Social only (messages)
  2) Block edits on conflict (hook denies) + social for everything else
  3) Force worktrees per agent always
- **Decision:** Enforce deterministically at pre-edit time (hooks / wrapper `check`), but keep default workflow social.
- **Consequences / follow-ups:**
  - Blocking must have satisfiable exits (claims TTL/release, explicit “go ahead”).

## DD-0003: Keep the instruction set tiny + identity framed
- **Context:** Long rule lists get ignored; models habituate to static text.
- **Options considered:**
  1) Long “rules” doc
  2) 3 core behaviors + values framing
- **Decision:** Reduce to three core behaviors (announce, listen, ask-before-touching) and phrase as “who you are” rather than “rules.”
- **Consequences / follow-ups:**
  - Hooks should include *both* a short repeated nudge and live/novel state.

## DD-0004: Start with harness Case 01 and keep scope minimal
- **Context:** The coordination tool has many potential features (messages, richer policies, UI), but correctness hinges on a single core property: an active claim by agent A must block agent B at a danger point (`check`).
- **Options considered:**
  1) Implement the full spec up front
  2) Implement only what Case 01 requires, then expand case-by-case
- **Decision:** Begin with Case 01 and implement only `--agent-id`, `claim`, and `check` with a repo-scoped SQLite lease table.
- **Consequences / follow-ups:**
  - Avoids locking in premature schema/UX decisions.
  - Forces additive design via new harness cases (expiry, habit formation, messaging) as they become required.
