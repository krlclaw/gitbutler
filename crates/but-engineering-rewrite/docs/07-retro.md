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
- Added a new 3-agent harness case that combines: (1) triangle claim conflicts (advisory vs `--strict`), (2) a provider/consumer dependency chain (B depends on A's API declaration), (3) a third agent with overlapping tokens but different scope that must not receive dependency hints, and (4) hint dedupe/noise control: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/05i-three-agent-triangle-deps.md`.
- Confirmed `check` continues to emit `dependency_hints` even when blocked (warn/deny), so dependency coordination remains visible alongside claim-based coordination: `crates/but-engineering-rewrite/harness/run.sh`.
- Extended harness coverage to assert dependency-hint scope filtering prevents cross-component token collisions even when another agent posts an API-tagged declaration for the colliding token: `crates/but-engineering-rewrite/harness/run.sh`.

### (b) What Was Hard
- The hint heuristic is intentionally lightweight and “stringly”; adding scope filtering is cheap, but it also exposes that we should define whether scope matching is exact, hierarchical, or tag-based long-term.

### (c) Next Step
- Extend the harness to cover multiple intents/declarations per agent and require deduping hints (one per provider agent + scope), rather than emitting one hint per overlapping declaration row.

## 2026-02-14 (Case 05e)

### (a) What Changed
- Added harness Case 05e to require `dependency_hints` to dedupe repeated declarations from the same provider (one hint per provider+scope): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/05e-dependency-hint-dedupe.md`.
- Updated both implementations to keep only the newest hint per `(provider_agent_id, scope)` to avoid noisy repeats during “renewal”/iteration: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- Dedupe has to happen after confirming an actual overlap; otherwise a newer (non-overlapping) declaration could accidentally suppress an older overlapping one.

### (c) Next Step
- Extend the dedupe key to include a stable “surface id” namespace (not raw tokens), and consider emitting a single aggregated hint per provider with unioned overlap tokens across declarations.

## 2026-02-14 (Case 10)

### (a) What Changed
- Added harness Case 10 to require basic visibility into active work via `claims` (list active leases with `path` + `agent_id`): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/10-claims-list.md`.
- Implemented minimal `claims` command in both the Rust CLI and the harness stub so the harness behaves consistently with/without Rust tooling: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- Keeping output shape aligned between Rust and the stub; even “simple” list commands become two implementations because the harness can run either binary.

### (c) Next Step
- Decide whether `claims` should support filtering (by prefix/path) and whether `check` should embed claim metadata (expiry) so wrappers can avoid extra round trips.

## 2026-02-14 (Case 11)

### (a) What Changed
- Ensured discovery `brief` preserves provenance by including `agent_id` on each propagated discovery (so consumers can follow up with the right agent): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/11-discovery-provenance.md`.
- Updated the harness stub implementation to attach `agent_id` (and to derive `next_steps` from the same filtered set) using Python JSON parsing instead of brittle `sed`/`awk` extraction: `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- The stub previously treated discoveries as raw JSON strings and relied on regex extraction; once we needed to inject provenance, regex parsing broke on nested arrays like `evidence[]`.

### (c) Next Step
- Consider factoring “load discoveries + derive next steps” into a shared helper or script so behavior stays consistent across the Rust CLI and the stub as cases expand.

## 2026-02-14 (Case 12)

### (a) What Changed
- Added harness Case 12 to require discovery provenance in `digest` output too (digest should be concise, not anonymous): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/12-discovery-provenance-digest.md`.
- The stub already emits `agent_id` in digest discoveries after the Case 11 fix, so no additional CLI surface was needed beyond adding the harness case.

### (b) What Was Hard
- Keeping the case realistic while still minimal: digest is intentionally compact, so the assertion focuses on preserving `agent_id` and `title` rather than the full discovery payload.

### (c) Next Step
- Decide whether `digest` should include a stable discovery id (row id / hash) so consumers can request full details on-demand without re-scanning.

## 2026-02-14 (Case 13)

### (a) What Changed
- Added harness Case 13 to require minimal “who is doing what” visibility via `status`, `plan`, and `agents`: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/13-agents-status-plan.md`.
- Implemented `status <text...> | status --clear`, `plan <text...> | plan --clear`, and `agents` in the Rust CLI, backed by a tiny repo-scoped SQLite `agent_state` table: `crates/but-engineering-rewrite/src/main.rs`.
- Updated the harness stub to support the same commands and share a single schema init path (claims/messages/agent_state) so the harness behaves consistently with/without Rust tooling: `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- The harness can run either the Rust binary or the stub; adding even “simple” new CLI surface effectively means two implementations plus keeping the SQLite schema compatible.
- Shell + embedded Python is easy to break with tiny formatting mistakes (indentation inside `python3 -c` blocks caused an early harness failure before the schema/init was centralized).

### (c) Next Step
- Make `agents` reflect liveness better (expire agents with no recent activity), and teach `eval user-prompt-submit` to show a compact “active agents + statuses” summary (not just claim count).

## 2026-02-14 (Case 05f)

### (a) What Changed
- Added harness Case 05f to require “claim renewal” semantics: re-claiming the same `(agent_id, path)` must not create duplicate rows visible via `claims` (keeps coordination state low-noise): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/05f-claim-renewal-dedupe.md`.
- Implemented renewal-by-dedupe in both implementations by deleting the existing `(agent_id, path)` row before inserting the renewed lease: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- SQLite schema migration would be “cleaner” with a unique constraint, but that’s more complexity than needed for the harness; doing a targeted delete keeps the behavior correct with minimal surface area.

### (c) Next Step
- Consider upgrading `claims` storage to a real upsert (`UNIQUE(path, agent_id)` + `ON CONFLICT DO UPDATE`) once we want stronger invariants beyond the disposable-repo harness.

## 2026-02-14 (Case 18)

- Added harness Case 18 to ensure `claims` is a useful coordination view by filtering out expired leases: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/18-expired-claims-filtered.md`.
- Kept the scenario focused on the operator workflow (someone runs `claims` to decide whether to proceed), rather than re-testing `check` behavior.
- Reinforced that time-based state must not linger in "listing" surfaces, otherwise people will ping the wrong agent and avoid safe work.
- Next: consider adding an explicit `claims --all` (include expired) for debugging, while keeping the default view clean.

## 2026-02-14 (Case 05g)

### (a) What Changed
- Added harness Case 05g to require a single "I'm done" cleanup path: release all of the agent’s active claims, clear `status`/`plan`, and post a completion message to the shared channel: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/05g-done-cleanup.md`.
- Implemented a minimal `done <summary...>` command in both the Rust CLI and the stub so the harness passes regardless of Rust tool availability: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- Getting a robust harness assertion for "status/plan cleared" without JSON parsing in bash; the case asserts the previous values are no longer present in `agents` output instead of matching nested structure.

### (c) Next Step
- Decide whether `done` should support `--no-post` and/or `--release --path ...` variants, and whether completion messages should be a first-class kind (e.g. `kind: done`) rather than a plain channel message.

## 2026-02-14 (Case 14)

### (a) What Changed
- Added harness Case 14 to require a filtered claims view via `claims --path-prefix <path>` so wrappers can show only relevant active work for a file/subtree (including overlapping directory claims): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/14-claims-filter.md`.
- Implemented minimal `--path-prefix` support in both the Rust CLI and the harness stub using the same overlap predicate as `check` (exact match OR ancestor/descendant on segment boundaries): `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- Getting “filter” semantics right: naive `starts_with` misses the important directory-claim overlap case (claim `src/` should surface when filtering `src/app.txt`).

### (c) Next Step
- Add richer `check` output that references the specific claim rows (path + agent + expiry) so wrappers can render conflicts without a separate `claims` query.

## 2026-02-14 (Case 15)

### (a) What Changed
- Added harness Case 15 to cover a realistic “multiple blockers” situation: two different agents can both overlap a target path (e.g. one claims `src/` while another claims `src/app.txt`). `check` must list each blocker once and produce an action plan that references each blocking agent: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/15-multi-blocker-action-plan.md`.
- Updated `check` to dedupe `blocking_agents` and to emit one “ping the blocker” step per blocking agent (instead of only the first): `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- The harness can run either the Rust binary or the stub; “simple” output-shape changes like multi-blocker action plans require updating both implementations to avoid flaky environment-dependent behavior.

### (c) Next Step
- Add structured `action_plan` items (kind + params) so wrappers can render/run steps safely without shell-escaping concerns.

## 2026-02-14 (Case 16)

### (a) What Changed
- Added harness Case 16 to make `read` ergonomic by defaulting to the shared channel transcript (messages) when no `--type` is provided: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/16-read-default-transcript.md`.
- Updated both the Rust CLI and the harness stub so `but-engineering-rewrite --agent-id X read` returns `kind:"message"` (instead of an effectively-empty `all` kind): `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- The harness can run either the Rust binary or the stub; a "default behavior" change needs to be implemented twice to avoid environment-dependent failures.

### (c) Next Step
- Decide what `read --type all` should mean (single unified schema vs per-kind lists) so “see everything” doesn’t silently drop non-discovery kinds.

## 2026-02-14 (Case 17)

### (a) What Changed
- Added harness Case 17 to require an explicit “escape hatch” for discovery inspection: `brief --all` must include low-signal discoveries (while the default `brief` remains high-signal-only): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/17-brief-all-escape-hatch.md`.
- Implemented minimal `--all` support for `brief`/`digest` in both implementations (Rust CLI + harness stub), and ensured `next_steps` is derived from the same filtered (or unfiltered) discovery set: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- Keeping CLI behavior aligned across the Rust binary and the stub while adding a new flag (the harness may execute either).

### (c) Next Step
- Add `--min-signal <low|high>` (default `high`) so callers can choose behavior without a boolean that’s discovery-specific, and extend the same filtering semantics to `read --type discovery` (e.g. `read --min-signal high`).

## 2026-02-14 (Case 05h)

### (a) What Changed
- Added harness Case 05h to prevent dependency-hint false positives caused by naive substring matching on tags (e.g. tag `capistrano` contains `api` but is not an API declaration): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/05h-dependency-hint-tag-gate.md`.
- Tightened “API-ish declaration” detection to require an `api` tag segment (split on non-alphanumeric) instead of `contains("api")`, in both implementations so the harness behaves consistently with/without Rust tooling: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

### (b) What Was Hard
- The harness can execute either the Rust binary or the stub, so even a tiny heuristic fix has to land twice to avoid environment-dependent failures.

### (c) Next Step
- Define an explicit tag vocabulary (e.g. `component/api` exactly) and validate it on `post --type declaration` to keep hinting deterministic and reduce heuristic creep.
