# Retro Notes

## 2026-02-14

### (a) What Changed
- Added harness Case 04 coverage for “high-signal discovery propagation” via a dedicated `brief` command (`crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/04-high-signal-discovery.md`).
- Tightened the Case 04 harness to assert `next_steps` actually contains the command from the posted discovery payload (not just somewhere in the output).
- Fixed the stub CLI to derive `next_steps` from the posted discovery’s `suggested_action.cmd` (so Case 04 passes when Rust tooling is absent): `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Made the Case 04 `grep` regex portable across variants (BSD `grep` did not like `[^\]]`): `crates/but-engineering-rewrite/harness/run.sh`.

### (b) What Was Hard
- The harness can run either the Rust binary or the stub depending on tool availability, so Case 04 needed behavior-alignment in both implementations (especially around deriving `next_steps`).
- Shell portability gotchas: `sed -E` escaping and `grep -E` bracket expressions vary enough to trip “looks-correct” regexes.

### (c) Next Step
- Add a “high-signal” gate: require (and validate) minimum fields for discoveries (title + evidence + suggested_action), and teach `digest` to prioritize or dedupe discoveries rather than listing everything forever.

## 2026-02-14 (Follow-up)

### (a) What Changed
- Made Case 04 more realistic by posting both low- and high-signal discoveries and asserting `brief`/`digest` only propagate the high-signal one (and that `next_steps` is derived from it): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/04-high-signal-discovery.md`.
- Added a minimal high-signal gate to both implementations so behavior matches whether the harness runs the Rust binary or the stub: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Added minimal discovery payload validation on `post --type discovery` (title + evidence + suggested_action.cmd): `crates/but-engineering-rewrite/src/main.rs`.

### (b) What Was Hard
- Keeping the Rust binary and shell stub aligned without pulling in a real JSON parser in the stub (the stub’s gate is intentionally string-based and assumes compact JSON).

### (c) Next Step
- Add an escape hatch to view everything (e.g. `brief --all`), and add a `min_signal` concept so “high only” stays the default but is configurable.

## 2026-02-14 (Harness Reliability + Dependency Hints)

### (a) What Changed
- Updated the harness to rebuild the default debug binary on each run (avoids stale-binary false negatives while iterating on cases): `crates/but-engineering-rewrite/harness/run.sh`.
- Added Case 05 (“dependency hints”) to the harness and implemented minimal `dependency_hints` emission on `check` based on intent/declaration surface overlap (no locking): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/05-dependency-hint.md`, `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- The harness previously reused an existing `target/debug` binary without rebuilding, which made behavior diverge from source changes and produced misleading failures.

### (c) Next Step
- Make the hinting heuristic less “latest-message” biased (aggregate multiple intents/declarations and dedupe), and add a minimal `read --type intent|declaration` to inspect what’s driving hints.

## 2026-02-14 (Case 05)

### (a) What Changed
- Added harness Case 05 for “dependency hint” (provider/consumer API): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/05-dependency-hint.md`.
- Extended `post --type ... --json ...` to support two new structured message types: `declaration` (scope + tags + surface) and `intent` (scope + tags + surface).
- `check` now emits `dependency_hints` when the caller’s latest intent overlaps another agent’s declared API surface (heuristic: surface token intersection + `api` tag), without changing the allow/warn/deny decision: `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`, `crates/but-engineering-rewrite/src/main.rs`.

### (b) What Was Hard
- Portability: the harness stub cannot assume `jq`, and BSD `sed`/`grep` regex differences made string-parsing JSON too fragile; switched the stub dependency-hint parsing to a tiny `python3 -c` snippet.

### (c) Next Step
- Make hints more precise: include a stable “surface id” namespace (e.g. `api:SyncService::push`) and dedupe hints (one per provider agent), plus add a `read --type declaration|intent` so agents can inspect current declared surfaces directly.

## 2026-02-14 (Case 06)

### (a) What Changed
- Added harness Case 06 to ensure agents can inspect the structured state driving dependency hints via `read --type declaration|intent`: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/06-read-surfaces.md`.
- Implemented minimal `read --type declaration|intent` support (with `agent_id` provenance) in both the Rust CLI and the stub so the harness behaves consistently with/without Rust tooling: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- Keeping output shape aligned between the Rust binary and the stub: the Rust path parses/augments JSON properly, while the stub has to assemble merged objects without `jq` (so it uses a tiny `python3 -c` helper).

### (c) Next Step
- Decide whether `brief` should also support `--type intent|declaration` (or a unified `read` schema across kinds) so wrappers can render a single “state view” without special-casing.

## 2026-02-14 (Case 07)

### (a) What Changed
- Added harness Case 07 (“release claim”) to ensure an agent can explicitly unblock others after finishing work (TTL alone is not a great UX): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/07-release-claim.md`.
- Implemented minimal `release --path <path>` support in both implementations by deleting the caller’s claim row for that path: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- Keeping behavior consistent across the Rust binary and the harness stub (the harness may run either depending on tool availability).

### (c) Next Step
- Add richer conflict output (e.g. who holds the lease + expiry time) and/or `release --all` to support “I’m done” cleanup without enumerating paths.

## 2026-02-14 (Case 08)

### (a) What Changed
- Added harness Case 08 to ensure collision detection triggers on directory/file prefix overlap (claim `src/` should conflict with check `src/app.txt`): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/08-path-prefix-overlap.md`.
- Extended `check` to treat ancestor/descendant paths as overlapping (exact match OR `a/b` under `a` OR vice versa), and normalized claim/check paths by trimming leading `./` and trailing `/`: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- Using SQLite string matching without a real path schema is easy to get subtly wrong (prefix collisions like `src` vs `src2`). The harness forces the safe “prefix + separator” behavior.

### (c) Next Step
- Decide whether directory claims should be explicit (`--recursive`) rather than implicit via prefix overlap, and add path canonicalization rules (case sensitivity, `..`, symlinks) once we move beyond the disposable-repo harness.

## 2026-02-14 (Case 05b)

### (a) What Changed
- Added harness Case 05b to require `check` to be directly machine-actionable when blocked: it must name `blocking_agents` and include an `action_plan` that references the blocking agent via `@agent` (coordination without hard locks): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/05b-check-action-plan.md`.
- Extended `check` output in both implementations to include `blocking_agents` (derived from conflicting claims) and a minimal `action_plan` (read, ping blocker, retry): `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- Keeping the stub and Rust output shape aligned: the harness can run either, so adding fields needs to be done in both places.

### (c) Next Step
- Move `action_plan` from a list of strings to a stable structured schema (kind + params) so wrappers can render/run safely without shell-escaping issues.

## 2026-02-14 (Case 09)

### (a) What Changed
- Added harness Case 09 to require basic “channel transcript” functionality: plain `post <text>` must be persisted and `read --type message` must return messages with provenance (`agent_id`): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/09-channel-messages.md`.
- Implemented minimal message persistence for `post <text>` in both implementations (Rust binary and harness stub) so the harness passes regardless of Rust tool availability: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- Shell quoting for SQL inserts is deceptively fragile once messages contain apostrophes; the stub now uses Python’s sqlite bindings for message inserts to avoid portability/escaping bugs.

### (c) Next Step
- Decide whether `read` should support a “since” cursor (id/time) so wrappers can show unread messages without re-rendering the full transcript forever.

## 2026-02-14 (Case 05d)

### (a) What Changed
- Added harness Case 05d to make dependency hints less noisy by requiring intent/declaration scope match (prevents cross-component token collisions): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/05d-dependency-hint-scope-filter.md`.
- Updated both the Rust CLI and the harness stub to filter dependency hints by `scope` equality in addition to surface token overlap + `api` tag: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- The hint heuristic is intentionally lightweight and “stringly”; adding scope filtering is cheap, but it also exposes that we should define whether scope matching is exact, hierarchical, or tag-based long-term.

### (c) Next Step
- Extend the harness to cover multiple intents/declarations per agent and require deduping hints (one per provider agent + scope), rather than emitting one hint per overlapping declaration row.
