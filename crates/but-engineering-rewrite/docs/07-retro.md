# Retro Notes

## 2026-02-15

- Extended the E2E scaffold with a minimal E2E-02 slice (`--scenario discovery`) that posts a valid structured discovery, asserts it appears in `brief`/`digest`, and ensures the suggested action is executed (posts an ack): `crates/but-engineering-rewrite/e2e/run.sh`.
- Added a pinned Codex prompt for the discovery scenario to keep the real-agent path deterministic and verdictable via a sentinel token: `crates/but-engineering-rewrite/e2e/prompts/discovery.step1.codex.txt`.
- Re-ran the fast harness after the E2E changes to ensure no regressions before checkpointing: `./crates/but-engineering-rewrite/harness/run.sh`.
- Grew the harness into a more realistic "closed-loop coordination" suite by adding cases around unread relevant updates, ack suggestions (dedupe + anti-ping-pong), and suppressing redundant blocker pings: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/`.
- Kept Rust and the harness stub in parity by mirroring the coordination heuristics (closure/ack parsing, unread relevance matching, and action plan shaping) in both implementations: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Added an opt-in E2E scaffold with a deterministic CLI smoke scenario plus optional real-agent spawn (Codex/Claude), with timeboxing, transcripts, and a structured JSONL trace: `crates/but-engineering-rewrite/e2e/run.sh`, `crates/but-engineering-rewrite/e2e/prompts/`.
- Extended the E2E scaffold with a minimal E2E-01 “collision -> read/ack -> release -> proceed” slice (`--scenario collision`), including pinned Codex prompts and a per-scenario sentinel token for deterministic verdicts: `crates/but-engineering-rewrite/e2e/run.sh`, `crates/but-engineering-rewrite/e2e/prompts/collision.step*.codex.txt`.
- Captured the working priorities in a living backlog doc so the next slices are obvious: `crates/but-engineering-rewrite/docs/08-backlog.md`.
- Harness remained green after the above changes, making this a safe checkpoint: `./crates/but-engineering-rewrite/harness/run.sh`.

## 2026-02-14

- Added harness Case 42 to prevent "ack ping-pong" loops: when an unread relevant update is itself the exact auto-ack template (`@X: ack: saw your update re <path>.`), `check --path` must not suggest acknowledging it back: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/42-ack-loop-suppression-auto-ack.md`.
- Improved coordination usefulness for allow decisions by removing the redundant "re-run check" step from `action_plan` when there are no blockers (the `check` already happened): `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Kept behavior aligned between Rust and the harness stub by applying the same auto-ack suppression predicate during ack-enrichment in both implementations: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

- Added harness Case 40 to ensure closure semantics still work under contention: when `check --strict` denies due to a blocking claim, it should still surface unread relevant updates from non-blocking agents and suggest a single `@X: ack: ...` step: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/40-strict-deny-ack-unread-updates-nonblocker.md`.
- Asserted the ack suggestion is scoped correctly: ack the non-blocking updater (`C`) but do not ack the blocker (`A`), since blockers already have explicit coordination steps in the conflict plan: `crates/but-engineering-rewrite/harness/run.sh`.
- No implementation changes were required; both the Rust CLI and the stub already enrich `action_plan` with deduped acks based on unread relevant updates, independent of allow/warn/deny decisions.

- Added harness Case 41 to cover a common miscommunication+repair loop: a broad directory claim blocks a teammate, the teammate asks for clarification, the blocker releases, and the checker sees the repair message as an unread relevant update: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/41-miscommunication-repair-release-and-ack.md`.
- Locked in closure semantics after repair: once unblocked, `check --path` should suggest a single explicit `@A: ack: ...` back to the repairing agent, and should not keep suggesting the original blocker-ping step once there are no blockers: `crates/but-engineering-rewrite/harness/run.sh`.
- Kept the scenario anti-spam by asserting a third `check` does not repeat the same repair message or the ack suggestion (cursor advanced): `crates/but-engineering-rewrite/harness/run.sh`.

- Added harness Case 39 to lock in closed-loop coordination for unread updates: when `check --path` surfaces an unread relevant update, it should suggest an explicit `post "@X: ack: ..."` step, and that suggestion must not repeat once the cursor advances: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/39-ack-unread-updates.md`.
- Extended the Rust CLI `check` action plan to include a deduped ack step per update-author (excluding blocking agents to avoid double-pinging), keeping the behavior additive and low-noise: `crates/but-engineering-rewrite/src/main.rs`.
- Updated the harness stub CLI to mirror the same ack-enrichment behavior so harness results are consistent when Rust tooling is unavailable: `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Kept the semantics anti-spam by relying on the existing unread-cursor mechanism: once an update is surfaced, subsequent `check` calls do not re-suggest the ack for the same message window.

- Added harness Case 38 to lock in a coordination-quality behavior: `check --path` should include the blocking agent's `status`/`plan` snapshot so callers do not need an extra `agents` round-trip when blocked: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/38-blocking-agent-status-plan.md`.
- Extended the Rust CLI `check` JSON output with an additive `blocking_agents_state` field (status/plan/updated_at) for each blocking agent: `crates/but-engineering-rewrite/src/main.rs`.
- Updated the harness stub CLI to emit the same `blocking_agents_state` field so the harness remains consistent when Rust tooling is unavailable: `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Fixed the Case 38 harness assertion helper script to avoid a Python f-string quoting pitfall (keeps the JSON-shape check readable and portable): `crates/but-engineering-rewrite/harness/run.sh`.

- Added a new coordination-compliance harness family (Cases 35-36) to force closed-loop behavioral dynamics: (1) staleness detection for blocking agents with actionable follow-up, and (2) unread relevant update deltas that do not repeat once “seen”: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/35-status-plan-ttl-staleness.md`, `crates/but-engineering-rewrite/harness/cases/36-check-unread-relevant-updates-cursor.md`.
- Extended `check --path` JSON output with additive fields: `stale_agents` (explicit stale indicators + suggested `post` command) and `unread_relevant_updates*` (label, cursor, and update payloads), keeping the core hookless and machine-consumable: `crates/but-engineering-rewrite/src/main.rs`.
- Added minimal persistent cursor state in SQLite (`agent_cursors`) to track per-agent last-seen message id per check topic (`check_path:<path>`), so a second `check` does not repeat the same relevant transcript items: `crates/but-engineering-rewrite/src/main.rs`.
- Added `COORD_STALE_SECONDS` env var to let the harness force quick staleness in tests (2s) without changing defaults for real use: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/run.sh`.
- Updated the harness stub CLI to mirror the new DB table and `check` JSON fields so the harness stays consistent even when Rust tooling is unavailable: `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Added harness Case 37 to lock in “directory mention” relevance: if an agent posts about `src/`, a `check --path src/app.txt` should surface that unread note as relevant: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/37-check-unread-updates-parent-dir-overlap.md`.
- Updated unread-update relevance matching to treat any ancestor directory as overlapping (not only exact file path), matching how humans coordinate in practice: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Made the unread cursor advance based on the max seen message id (even if none are relevant) so repeated `check` calls do not re-scan the same transcript window: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

- Added harness Case 34 to ensure `check --path` collapses `..` segments (e.g. `src/../notes.txt`), preventing false-positive claim overlaps that would otherwise create needless coordination churn: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/34-path-normalization-dotdot.md`.
- Extended the Rust CLI path normalizer to collapse `.`/`..` segments in a filesystem-free way (string-only stack), keeping overlap checks stable for common wrapper-produced relative paths: `crates/but-engineering-rewrite/src/main.rs`.
- Updated the harness stub CLI to use the same `.`/`..` collapsing normalization so behavior stays aligned when Rust tooling is unavailable: `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Kept the scenario narrowly focused on coordination correctness (avoiding an incorrect `claimed_by_other` decision), without attempting full canonicalization or filesystem resolution.

- Added harness Case 33 to ensure `release --path` normalizes trailing slashes (e.g. releasing `src/` should clear a claim stored as `src`), preventing stale leases due to common path spelling differences: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/33-release-path-normalization-trailing-slash.md`.
- Kept the scenario focused on coordination ergonomics for wrappers: a successful release must immediately unblock `check` for another agent on a descendant path.
- No CLI changes were required for this iteration; existing path normalization already trims trailing `/` consistently for `claim` and `release`: `crates/but-engineering-rewrite/src/main.rs`.

- Added harness Case 32 to lock in a coordination-quality behavior: even when `check` returns `allow`, it should still emit low-noise FYI steps for other agents with active (non-overlapping) claims, so wrappers can show “who’s working on what” without forcing a conflict: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/32-allow-includes-nonblocking-fyi.md`.
- Asserted the FYI entry does not suggest releasing unrelated claims (avoid churn), while still mentioning both the other agent’s active claim path and the checked path for clarity: `crates/but-engineering-rewrite/harness/run.sh`.
- No CLI changes were required for this iteration; the existing `action_plan_by_agent` behavior already emits FYI steps for active non-blocking claims even on `allow`: `crates/but-engineering-rewrite/src/main.rs`.

- Added harness Case 31 to ensure `claims --path-prefix` normalizes common spellings like a leading `./` (matching `check`/`release` path normalization), keeping coordination wrappers from needing bespoke path cleanup: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/31-claims-filter-path-normalization-dot-prefix.md`.
- Confirmed the Rust CLI already normalizes `claims --path-prefix` during arg parsing, so the new scenario passed without further implementation work: `crates/but-engineering-rewrite/src/main.rs`.
- This case specifically targets the filtered coordination view (not conflict detection) to avoid duplicating the existing `check`/`release` path-normalization cases while still locking in wrapper ergonomics.

- Added harness Case 30 to lock in a more coordination-useful `check` surface: `blocking_claim_paths_by_agent` groups overlapping claim paths per blocker (most-specific first), so wrappers don’t need to regroup the flat list client-side: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/30-blocking-claim-paths-by-agent.md`.
- Extended the Rust CLI `check` output to include `blocking_claim_paths_by_agent` alongside the existing `blocking_claims` list: `crates/but-engineering-rewrite/src/main.rs`.
- Extended the harness stub CLI to emit `blocking_claim_paths_by_agent` as well, keeping behavior aligned when Rust tooling is unavailable: `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Kept the scenario realistic by covering the common “directory + file claim by the same blocker” case, and asserting stable ordering and dedupe (most-specific first): `crates/but-engineering-rewrite/harness/run.sh`.

- Added harness Case 29 to make `check` more coordination-useful when a blocker holds multiple overlapping claims: the per-agent plan should suggest releasing the most specific claim first (avoid releasing a broad directory claim if a file claim would unblock): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/29-action-plan-release-most-specific.md`.
- Updated the stub CLI’s `action_plan_by_agent` to choose the most-specific overlapping claim path per blocker (longest path; tie-break on expiry) so suggested releases are low-churn: `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Updated the Rust CLI to mirror this behavior, and to avoid emitting `action_plan_by_agent` entries for agents with no active claims (reduces noise and matches existing harness expectations): `crates/but-engineering-rewrite/src/main.rs`.

- Added harness Case 28 to keep `check` conflict output actionable when a blocker holds multiple overlapping claims (directory + file): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/28-multiple-blocking-claims-per-agent.md`.
- Updated the stub CLI `blocking_claims` emission to include all overlapping claim paths per blocking agent (deduped), not just a single "best" path: `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Mirrored the `blocking_claims` surface in the Rust CLI so behavior stays aligned when Rust tooling is available: `crates/but-engineering-rewrite/src/main.rs`.

- Fixed a brittle harness assertion in Case 26 (contraction mismatch) so it matches the stub output text: `crates/but-engineering-rewrite/harness/run.sh`.
- Added harness Case 27 to lock in a coordination-quality behavior: `check` should not emit FYI steps for agents with no active claims (reduces stale/noise in `action_plan_by_agent`): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/27-check-ignores-expired-nonblocking-claims.md`.
- Updated the stub CLI to skip emitting per-agent `action_plan_by_agent` entries for agents without active claims (still includes blockers and non-blocking agents with active claims): `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.

- Added harness Case 26 to ensure `check` remains coordination-useful even for non-blocking agents: it should include a FYI step for agents with active claims on other paths (helps teams avoid duplicate work without forcing releases).
- The case asserts we do not suggest unrelated cleanup (non-blocking agents should not be told to `release` their non-overlapping claim).
- No CLI changes were needed; the existing `action_plan_by_agent` behavior already covered this branch.
- Files: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/26-nonblocking-agent-fyi-active-claim.md`.

- Added harness Case 25 to cover the common workflow of checking a directory (`check --path src/`) when another agent holds a file claim inside it (reverse prefix overlap): `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/25-path-prefix-overlap-directory-check.md`.
- Asserted the conflict output stays actionable by including the specific blocking file claim in `blocking_claims` (not just the blocking agent id).
- No CLI changes were required for this iteration; existing overlap detection and `blocking_claims` output already covered the scenario.

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

## 2026-02-14 (Case 24)

### (a) What Changed
- Added harness Case 24 to cover a realistic coordination footgun: releasing a claim with a leading `./` path spelling should still unblock others: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/24-release-path-normalization-dot-prefix.md`.
- Extended the harness runner list to execute Case 24.

### (b) What Was Hard
- Avoiding a near-duplicate of Case 23 (which covers `check` normalization) while still testing a separate high-impact workflow (`release`).

### (c) Next Step
- Expand path normalization beyond leading `./` (e.g. redundant slashes and `..`) if/when we need cross-platform robustness.

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

## 2026-02-14 (Case 21: Convergence Dynamics)

- Added a new harness case that simulates a 3-agent triangle conflict and a short convergence sequence where one agent narrows scope to reduce collisions: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/21-multi-step-convergence.md`.
- Extended `check` output to include `action_plan_by_agent` so wrappers can present concrete next actions for each involved agent (not just the caller): `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Added explicit graders for "collisions reduced" (blocker count drops) and "no spam" (deduped blocking agents and agent ids) using JSON parsing instead of brittle grep: `crates/but-engineering-rewrite/harness/run.sh`.
- Kept the convergence mechanism intentionally simple (release broad claim, re-claim narrower path) to model real coordination without introducing hard locks.

## 2026-02-14 (Case 19)

- Added harness Case 19 to ensure `status --clear` and `plan --clear` exist and prevent stale coordination signals: `crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/19-clear-status-plan.md`.
- Asserted clearing is field-scoped (clearing status preserves plan) and that the change is visible via `agents`.
- Kept the change harness-only; both implementations already support `--clear`, so no product code changes were needed this iteration.
- Reinforced a coordination invariant: stale status/plan is actively harmful, so explicit clearing must be first-class and easy.

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

## 2026-02-14 (Case 20)

- Added harness Case 20 to require per-message `created_at_ms` timestamps in the default transcript so clients can render "when" and sort deterministically (`crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/20-message-timestamps.md`).
- Updated the stub CLI `read` output to attach `created_at_ms` to each returned message object (derived from stored `created_at_s`) (`crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`).
- Fixed a Rust CLI bug where plain `post` discarded the bound `message` value in the match arm (`crates/but-engineering-rewrite/src/main.rs`).
- Kept the harness assertion non-flaky by only requiring timestamps to be non-decreasing (seconds-level clocks can collide) while still guaranteeing presence and order.
- Next: consider including `created_at_ms` consistently across `read`/`brief`/`digest` for discoveries too, so UIs can reason about freshness without custom rules.

## 2026-02-14 (Case 22)

- Added harness Case 22 to require `check` to expose `blocking_claims` (agent id + overlapping claim path + expiry), making conflicts directly actionable (`crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/22-blocking-claim-paths.md`).
- Extended the harness stub `check` JSON to compute and emit one "best" overlapping claim per blocking agent (prefer most specific path, then latest expiry) (`crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`).
- Kept the change additive (existing keys preserved) so older harness cases remain stable while coordination UIs get richer conflict details.

## 2026-02-14 (Case 23)

- Added harness Case 23 to assert path normalization for common spellings: `src/app.txt` and `./src/app.txt` must be treated as overlapping for coordination to be reliable (`crates/but-engineering-rewrite/harness/run.sh`, `crates/but-engineering-rewrite/harness/cases/23-path-normalization-dot-prefix.md`).
- This is a high-value “paper cut” scenario: agents frequently copy relative paths with `./`, and missed conflicts here are both silent and costly.
- Harness stayed green without implementation changes, which indicates the existing claim/check path handling already normalizes this prefix.
- Next: consider expanding normalization coverage in harness (e.g. `src//app.txt`, `src/./app.txt`, and `src/../src/app.txt`) and decide what should be supported vs rejected.
