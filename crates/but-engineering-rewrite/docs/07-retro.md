# Retro Notes

## 2026-02-16

- E2E replay metadata slice: `meta.json` now records replay consumption summary (`replay_expected_invocations`, `replay_consumed_invocations`, `replay_remaining_invocations`, `replay_complete`) so provider-trace replays are auditable without manually counting `trace.jsonl` rows: `crates/but-engineering-rewrite/e2e/run.sh`.
- Wired `replay_init` to capture the expected invocation total once and patched `meta.json` again after scenario execution, so final replay status reflects the run outcome rather than pre-scenario defaults.
- Why: this advances the top backlog item (replayable agent-mode E2E traces) with a small inspectability improvement and no replay matching semantic changes.
- Scope/parity: runner-only change; Rust CLI coordination logic and harness stub behavior remain untouched (`crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`).
- What failed/limits: this slice does not add stricter replay assertions or new scenario coverage; it only improves artifact-level replay diagnostics.

- E2E replay trace inspectability slice: each `trace.jsonl` invocation now records `replay_run`, `replay_short_circuit`, and `replay_expected_invocation_id` so agent-mode replay artifacts show which steps were actually replayed vs live-executed: `crates/but-engineering-rewrite/e2e/run.sh`.
- Why: the top backlog item is replayable provider-mode traces; this adds per-step provenance directly in the trace without changing replay pass/fail semantics.
- Scope/parity: runner-only additive fields; no Rust CLI coordination logic or harness stub behavior changed (`crates/but-engineering-rewrite/src/main.rs` and `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite` untouched).
- Verified: fast deterministic harness gate remains green after the runner change via `./crates/but-engineering-rewrite/harness/run.sh` (`Trial 1/1: PASS`).
- What failed/limits: this slice does not tighten replay matching rules; it improves artifact auditability only.

- E2E replay execution slice: in replay mode, `agent.*` steps are now short-circuited from saved `trace.jsonl` rows (reuse saved `stdout`/`stderr`/`exit_code`) instead of spawning live provider processes: `crates/but-engineering-rewrite/e2e/run.sh`.
- Replay-mode provider checks are now conditional, so `spawn_agent` does not require `codex`/`claude` binaries when replaying recorded agent steps: `crates/but-engineering-rewrite/e2e/run.sh`.
- Why: this closes a concrete gap in the top backlog item by making provider-run traces inspectable/replayable without re-running live agents.
- What failed + fix: replay against older traces initially failed on `stdin_file` basename mismatch (`smoke.codex.txt` vs `agent.smoke.prompt.txt`); replay compare now allows this legacy-vs-snapshot basename pattern for `agent.*` labels while still enforcing `stdin_sha256` when present: `crates/but-engineering-rewrite/e2e/run.sh`.
- Verified: targeted replay passes for both newer snapshot-style traces and older prompt-basename traces, and required fast harness gate remains green via `./crates/but-engineering-rewrite/harness/run.sh`.

- E2E replay inspectability slice: `trace.jsonl` rows now include `agent_event_summary` for `agent.*` steps (`types`, `turn_started`, `turn_completed`, `response_completed`, `jsonl_objects`) extracted from provider stdout JSONL: `crates/but-engineering-rewrite/e2e/run.sh`.
- Why: provider-mode traces were replayable but still tedious to inspect quickly; this adds stable, compact transcript shape metadata without requiring full transcript diffing.
- Scope/parity: runner-only change; no Rust CLI or harness-stub coordination behavior changed (`crates/but-engineering-rewrite/src/main.rs` and `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite` untouched).
- Verified gate: fast deterministic harness remains green after the edit via `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed/limits: replay matching semantics are unchanged in this slice (metadata is additive for debugging/auditability, not a stricter pass/fail gate yet).

- E2E replay hardening (prompt integrity): trace rows now include `stdin_sha256` (computed from `stdin_file`) so replay can verify agent prompt snapshot content, not just filename shape: `crates/but-engineering-rewrite/e2e/run.sh`.
- Replay compare now enforces `stdin_sha256` when it exists in the saved trace; older traces without this field still replay with basename-only checks for compatibility.
- Why: basename checks alone can miss prompt-content drift; hashing keeps provider-run traces more inspectable/replayable without requiring byte-for-byte transcript determinism.
- Scope/parity: runner-only change; no Rust CLI or harness stub coordination behavior changed (`crates/but-engineering-rewrite/src/main.rs` and `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite` untouched).
- What failed/limits: this still validates only prompt/input integrity and stable transcript markers, not full offline re-execution of live-agent side effects.

- E2E replay hardening (agent-mode provenance): replay now checks `stdin_file` for expected prompt-fed steps, requiring non-empty input and matching basename (for example `agent.step1.B.prompt.txt`) when the saved trace had one: `crates/but-engineering-rewrite/e2e/run.sh`.
- Why: replay already validated labels/exit-codes and some stdout shape, but prompt provenance could silently drift; this keeps replay inspectable for provider runs without overfitting absolute out-dir paths.
- Scope/parity: runner-only change; no Rust CLI coordination logic or harness stub behavior changed (`crates/but-engineering-rewrite/src/main.rs` and `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite` untouched).
- Verified gate: fast deterministic harness re-run remains required in this loop via `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed/limits: this still does not replay live-agent side effects offline; it tightens trace comparison only when provider steps are executed.

- E2E replay semantics hardening: `replay_compare` now adds an agent-mode hook for `agent.*` labels that inspects Codex-style JSONL output and requires stable event markers (`turn.started`, `turn.completed`, `response.completed`) present in the saved trace to also appear in the new run: `crates/but-engineering-rewrite/e2e/run.sh`.
- Why: previous replay checks for provider runs were mostly label + exit-code (and JSON object key checks when applicable), which was too weak for agent transcript regressions; this improves signal while staying tolerant to non-deterministic payload content.
- Scope/behavior: no Rust CLI or harness-stub coordination logic changed; this is runner-only and keeps parity untouched for `src/main.rs` vs `harness/bin/but-engineering-rewrite`.
- Verified gate: fast deterministic harness remains the hard checkpoint in this loop via `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed/limits: replay still does not rehydrate agent side effects without live agent execution; this slice only strengthens transcript-shape matching for runs that do execute provider steps.

- E2E replayability slice: agent prompts are now snapshotted into each run artifact directory (`$OUT_DIR/<label>.prompt.txt`) before provider execution, so traces can be inspected against the exact prompt text used at run time: `crates/but-engineering-rewrite/e2e/run.sh`.
- E2E runner now feeds Codex from the snapshot file (via `--stdin-file`) and feeds Claude from that same snapshot content, keeping provider behavior unchanged while making artifacts self-contained for agent-mode debugging/replay.
- Why: this is a minimal top-backlog increment toward replayable E2E traces for provider runs without changing coordination semantics or touching Rust/stub command behavior.
- Verified locally with the fast deterministic harness gate: `./crates/but-engineering-rewrite/harness/run.sh` (green in this iteration).
- What failed/limits: this slice does not strengthen replay matching semantics yet (it improves trace inspectability/provenance only).

- E2E runner: added replayable trace mode via `E2E_REPLAY_DIR=...` so new runs can be compared against a saved `trace.jsonl` without re-running live-agent behavior blindly: `crates/but-engineering-rewrite/e2e/run.sh`.
- Replay checks are intentionally lightweight for stability: per-invocation label + exit code must match, and when expected `stdout` is JSON object, new `stdout` must remain JSON with the same top-level keys.
- Added `invocation_id` to E2E trace rows and replay metadata (`replay`, `replay_dir`) in `meta.json` to make replay diagnostics inspectable in artifacts.
- Why: this is a small first slice of the current top backlog item (“replayable E2E traces for agent-mode runs”) while keeping deterministic harness CI as the hard gate.
- What failed: first replay attempt used `mapfile` (not available in this shell’s bash) and an invalid dual-stdin redirection pattern in Python; fixed with portable `while read` loading + argv-based compare input, then validated with `--no-agents` smoke replay.

- CI (ERW harness in main push workflow): made artifact upload strict with `if-no-files-found: error` so missing harness outputs fail loudly instead of passing with an empty upload: `.github/workflows/push.yaml`.
- Why: this is a small follow-up on the top backlog CI-integration priority; deterministic ERW coverage should fail fast on missing artifacts in both standalone and main workflows.
- Rust/stub parity: no coordination behavior change in this slice; `src/main.rs` and `harness/bin/but-engineering-rewrite` were not edited.
- Verified locally: fast harness re-run stayed green after the workflow-only tweak: `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed: GitHub Actions cannot be executed from this environment, so validation remains local harness + workflow diff inspection.

- Checkpoint: re-ran the fast harness and kept it green in this iteration (`Trial 1/1: PASS`), satisfying the top backlog priority to keep the loop healthy before taking larger slices: `./crates/but-engineering-rewrite/harness/run.sh`.
- Why: this workspace already had substantial in-flight changes across ERW/compare/CI files, so this slice intentionally avoided broad edits and focused on a safe checkpoint-first pass.
- Rust/stub parity: no behavior changes in this iteration; parity is preserved by leaving `src/main.rs` and `harness/bin/but-engineering-rewrite` untouched.
- What failed: did not attempt GitHub Actions or slow real-agent E2E runs from this environment; verification remains local harness execution only.

- CI (ERW harness): made harness artifact upload strict in the standalone deterministic workflow by adding `if-no-files-found: error`: `.github/workflows/test-but-engineering-rewrite-harness.yml`.
- Why: this is a small CI-integration hardening slice from the backlog so missing harness outputs fail loudly instead of producing a misleading green artifact step.
- Rust/stub parity: no CLI behavior changes in this iteration; Rust and harness stub remain unchanged.
- Verified locally: fast harness stayed green after workflow-only edits: `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed: GitHub Actions cannot be executed from this environment, so validation here is local harness pass + workflow diff inspection only.

- CI (ERW E2E): switched deterministic matrix runs to write artifacts to a scenario-scoped `--out-dir` under `${{ runner.temp }}` instead of the shared repo `e2e/out`, reducing cross-run artifact bleed and making each job’s output deterministic: `.github/workflows/push.yaml`, `.github/workflows/test-but-engineering-rewrite-e2e.yml`.
- CI (ERW E2E): made artifact upload strict with `if-no-files-found: error` so silent missing-output regressions fail fast instead of producing a green job with empty artifacts: `.github/workflows/push.yaml`, `.github/workflows/test-but-engineering-rewrite-e2e.yml`.
- Why: backlog priority is CI integration for regularly-run deterministic E2E; tightening artifact isolation and upload checks improves signal quality without touching coordination semantics.
- Verified locally: fast harness stayed green after workflow-only changes (no Rust/stub behavior changes): `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed: GitHub Actions jobs cannot be executed end-to-end in this environment, so CI validation here is limited to workflow inspection plus local harness pass.

- CI: added `erw-harness` to the main `push.yaml` workflow, gated on ERW path changes, so deterministic harness coverage runs in the same pipeline as ERW E2E: `.github/workflows/push.yaml`.
- CI: wired `erw-harness` into `check-rust` (`alls-green`) so ERW harness regressions can’t be silently bypassed when ERW files are touched: `.github/workflows/push.yaml`.
- Why: backlog priority is regular CI integration for the deterministic ERW suite; this keeps E2E matrix and fast harness both on the primary PR/push path.
- Verified locally: fast harness stayed green after the workflow-only change (no Rust/stub behavior changes): `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed: GitHub Actions execution can’t be run end-to-end in this local environment, so CI validation here is limited to YAML inspection + local harness pass.

- Check JSON: added `path` (normalized) plus a structured `next_steps` list (objects with `{cmd}`) to make scored consumers less string-parsey; kept Rust + harness stub in parity: `crates/but-engineering-rewrite/src/main.rs`, `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Why: downstream tooling/prompt code can treat `check.next_steps[*].cmd` as the canonical executable plan while preserving backwards compatibility with `action_plan`.
- Verified locally: fast harness stayed green after the additive JSON change: `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed: nothing new; this iteration is output-shape only (additive fields) and kept existing keys stable.

- Drift E2E: added a hard turn-count cap for Codex JSONL runs (counts `turn.started`) and enforce it for drift (<=6) and drift_v2 (<=10) to reduce infinite-loop risk: `crates/but-engineering-rewrite/e2e/run.sh`.
- Drift E2E: strengthened the drift step1 Codex prompt with additional irrelevant distraction content (more "long-lived session" noise): `crates/but-engineering-rewrite/e2e/prompts/drift.step1.codex.txt`.
- Why: the drift scenarios are meant to be anti-gaming and non-looping; a simple deterministic cap makes failures fast and debuggable instead of timebox-hanging.
- Verified locally: fast harness stayed green (no Rust/stub behavior changes): `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed: did not run the slow agent-spawned drift/drift_v2 E2E in this iteration; relied on harness + runner change review.

- Compare: added scored fast repeat runners for rewrite + legacy plus a tiny metrics helper (mean/stddev/min/max) so we can track variance across runs: `crates/but-engineering-compare/eval/package.json`, `crates/but-engineering-compare/eval/scripts/scored-repeat-metrics.mjs`.
- Why: repeatability and variance matter more than single-run point estimates for the scored benchmark; this makes it cheap to run `--repeat` and see whether a change is actually stable.
- Verified locally: fast harness stayed green (no Rust/stub behavior changes): `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed: did not execute the promptfoo scored repeat in this iteration (CI and local iteration stay harness-first); only added the runner plumbing.
- Compare: tightened `Skill(but-engineering-rewrite)` used by the scored rewrite prompt to explicitly forbid environment-probing commands (e.g. `which`, `--help`) and to clarify that `read` is coordination-state only (use `sed`/`cat` for file contents): `crates/but-engineering-compare/eval/skills/but-engineering-rewrite.SKILL.md`.
- Why: scored runs were wasting turns on tool thrash and on invalid “read the file via coordination CLI” attempts; the skill now pushes a deterministic per-file loop (check -> claim -> edit -> release).
- Verified locally: fast harness remained green after the skill-only change (no Rust/stub behavior changes): `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed: did not re-run the scored compare suite in this iteration; this is a prompt/skill-only tweak validated via harness.

## 2026-02-15

- Drift E2E: strengthened the `drift` step1 Codex prompt with extra intentionally-wrong distractions plus an explicit <=6-command budget to reduce “prompt-following” and nudge agents away from infinite loops: `crates/but-engineering-rewrite/e2e/prompts/drift.step1.codex.txt`.
- Verified locally: fast harness stayed green after the prompt-only change (no Rust/stub behavior changes): `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed: did not run the slow E2E agent-spawned drift scenario in this iteration; relied on harness + prompt review.

- CI: integrated deterministic ERW E2E into the main `push.yaml` workflow (matrix over `smoke..drift_v2`), gated on ERW file changes so it runs regularly without slowing unrelated PRs: `.github/workflows/push.yaml`.
- CI: wired the new ERW E2E job into the `check-rust` alls-green aggregator so it can’t be silently ignored when ERW changes are present: `.github/workflows/push.yaml`.
- Verified locally: fast harness stayed green after the workflow-only change (no Rust/stub behavior changes): `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed: local validation of GitHub Actions YAML structure is limited here (no `yq` available), so verification relies on harness + inspection.

- CI: extended the deterministic ERW E2E matrix to include `drift_v2` (the current stronger drift scenario) so CI covers it by default: `.github/workflows/test-but-engineering-rewrite-e2e.yml`.
- Verified locally: fast harness stayed green after the workflow-only change (no Rust/stub behavior changes): `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed: nothing new; this iteration only changes CI config and relies on local harness for verification.

- CI: made ERW harness + deterministic E2E runnable on demand (`workflow_dispatch`) and on a nightly schedule (catches regressions even when ERW files don’t change): `.github/workflows/test-but-engineering-rewrite-harness.yml`, `.github/workflows/test-but-engineering-rewrite-e2e.yml`.
- CI: added explicit least-privilege permissions and job timeouts to reduce stuck-run risk and clarify intent: `.github/workflows/test-but-engineering-rewrite-harness.yml`, `.github/workflows/test-but-engineering-rewrite-e2e.yml`.
- Verified locally: fast harness re-run stayed green after workflow-only changes; no Rust/stub behavior changes: `./crates/but-engineering-rewrite/harness/run.sh`.
- Added deterministic GitHub Actions coverage for ERW: fast harness + E2E matrix (`--no-agents`), with output uploaded as artifacts: `.github/workflows/test-but-engineering-rewrite-harness.yml`, `.github/workflows/test-but-engineering-rewrite-e2e.yml`.
- Tightened CI E2E invocation by dropping the redundant `--provider codex` under `--no-agents` and keeping a bounded timebox: `.github/workflows/test-but-engineering-rewrite-e2e.yml`.
- Strengthened the drift E2E prompt with explicit “long-lived session” distraction context and a “don’t loop forever” guardrail (nudges agents to re-orient via tool state vs prompt memory): `crates/but-engineering-rewrite/e2e/prompts/drift.step1.codex.txt`.
- Fixed the E2E runner’s unknown-scenario message to list `drift` as supported: `crates/but-engineering-rewrite/e2e/run.sh`.
- Verified locally with `e2e/run.sh --scenario smoke --no-agents` and kept the fast harness green; no Rust/stub behavior changes: `./crates/but-engineering-rewrite/e2e/run.sh`, `./crates/but-engineering-rewrite/harness/run.sh`.
- What failed: the fast harness doesn’t execute GitHub Actions or the slow E2E runner, so we relied on local smoke + harness coverage instead.

- Hardened E2E runner auto-build diagnostics: `cargo build` (when auto-building the default `target/debug/...` binary) is now executed via `run_with_timeout`, producing `cargo.build.{stdout,stderr}` artifacts plus a trace entry for post-mortem debugging: `crates/but-engineering-rewrite/e2e/run.sh`.
- Increased reliability of “slow suite” failures by making build timeouts less likely (minimum 600s timebox for the build step, independent of per-step CLI timeboxes): `crates/but-engineering-rewrite/e2e/run.sh`.
- No Rust/stub behavior changes; fast harness re-run remained green after the runner-only tweak: `./crates/but-engineering-rewrite/harness/run.sh`.

- Hardened E2E artifacts: `meta.json` now records the resolved `BIN` actually used (real vs stub), plus a best-effort `bin_sha256` so runs are replayable/auditable even when binaries change: `crates/but-engineering-rewrite/e2e/run.sh`.
- Reduced a `BIN=...` footgun: the runner now only auto-`cargo build` when `BIN` is the default `target/debug/...`; custom `BIN` paths fail fast with a clearer error (or can fall back via `--allow-stub`): `crates/but-engineering-rewrite/e2e/run.sh`.
- Verified the runner change with an offline smoke (`--no-agents`) and kept the fast harness green (no Rust/stub behavior changes): `./crates/but-engineering-rewrite/e2e/run.sh --scenario smoke --no-agents --allow-stub`, `./crates/but-engineering-rewrite/harness/run.sh`.

- Fixed an E2E runner `--only-step` footgun: `run_with_timeout` no longer `exit 0`s mid-run, so deterministic assertions still run and `verdict.json` still gets written: `crates/but-engineering-rewrite/e2e/run.sh`.
- Verified the new behavior via an offline smoke run that stops at `cli.check.B` while still producing artifacts: `./crates/but-engineering-rewrite/e2e/run.sh --scenario smoke --no-agents --allow-stub --only-step cli.check.B`.
- Harness stayed green after the runner-only change (no Rust/stub behavior changes): `./crates/but-engineering-rewrite/harness/run.sh`.
- Hardened E2E agent spawning determinism by snapshotting the exact prompt text into the run output dir and using that snapshot as the subprocess stdin/input (makes runs replayable even if prompt files change later): `crates/but-engineering-rewrite/e2e/run.sh`.
- Hardened E2E-01 `collision` determinism by asserting the initial `check` output includes a `read` step in `action_plan` (models “read before ping” and avoids relying solely on agent behavior): `crates/but-engineering-rewrite/e2e/run.sh`.
- No Rust/stub behavior changes; harness re-run stayed green after the runner-only hardening: `./crates/but-engineering-rewrite/harness/run.sh`.
- Implemented the previously-advertised `--only-step` option in the opt-in E2E runner: it now runs up to the first matching step label (prefix match) and stops early while still writing `trace.jsonl` and `verdict.json`: `crates/but-engineering-rewrite/e2e/run.sh`.
- Updated `--help` to match the new `--only-step` behavior and fail fast when the label matches no steps (exit 2): `crates/but-engineering-rewrite/e2e/run.sh`.
- No Rust/stub behavior changes; harness re-run remained green after the runner-only tweak: `./crates/but-engineering-rewrite/harness/run.sh`.
- Hardened the opt-in E2E runner ergonomics: `--provider both` is now allowed for non-`smoke` scenarios when `--no-agents` is set (provider is irrelevant in offline mode), reducing footguns in CI/local invocations: `crates/but-engineering-rewrite/e2e/run.sh`.
- Clarified `--help` text for the `both` provider constraint (now explicitly conditioned on `--no-agents`): `crates/but-engineering-rewrite/e2e/run.sh`.
- No Rust/stub behavior changes; harness remained green after the runner-only tweak: `./crates/but-engineering-rewrite/harness/run.sh`.
- Hardened the opt-in E2E runner CLI ergonomics by adding `--out-dir` (lets CI/local runs put artifacts in a known path rather than relying on timestamp discovery): `crates/but-engineering-rewrite/e2e/run.sh`.
- Added early validation for `--scenario`, `--provider`, and `--timebox-s` to fail fast with actionable errors (less time wasted debugging silent fallthroughs): `crates/but-engineering-rewrite/e2e/run.sh`.
- No Rust/stub behavior changes; harness remained green after the runner-only change: `./crates/but-engineering-rewrite/harness/run.sh`.
- Added E2E-03 `triangle` to the opt-in slow runner (mirrors harness Case 05i: 3-agent claim conflict + dependency-hint noise control): `crates/but-engineering-rewrite/e2e/run.sh`.
- Made `triangle` deterministic with JSON-shape assertions (B warn/deny, blocker sets, exactly-one provider-A hint, and no provider-C hint; C receives no hints): `crates/but-engineering-rewrite/e2e/run.sh`.
- Added a pinned Codex prompt + sentinel token so the runner can optionally spawn a real agent and still yield a deterministic verdict: `crates/but-engineering-rewrite/e2e/prompts/triangle.step1.codex.txt`, `crates/but-engineering-rewrite/e2e/run.sh`.
- Improved E2E artifacts with `meta.json` (git/tool versions) and `verdict.json` (pass/fail + exit code), and fixed Codex stdin piping so pinned prompts are reliably consumed: `crates/but-engineering-rewrite/e2e/run.sh`.
- Hardened the E2E runner failure mode for agent prompts: missing prompt files now produce a trace entry and a clear error (easier debugging vs a generic subprocess failure): `crates/but-engineering-rewrite/e2e/run.sh`.
- Patched E2E artifacts to include `repo_path` in `meta.json` and `verdict.json` (and `trace_path` in verdict) so failures can be mapped back to the temp repo quickly: `crates/but-engineering-rewrite/e2e/run.sh`.
- Harness stayed green (no Rust/stub behavior changes): `./crates/but-engineering-rewrite/harness/run.sh`.
- Hardened E2E `smoke` determinism by asserting `claimed_by_other` collisions include a `read` step in `action_plan` (models “read before ping” coordination): `crates/but-engineering-rewrite/e2e/run.sh`.
- Synced the backlog’s scenario list with the runner’s current supported scenarios (includes `triangle`): `crates/but-engineering-rewrite/docs/08-backlog.md`.
- Harness re-run stayed green after the E2E assertion change (no Rust/stub behavior changes): `./crates/but-engineering-rewrite/harness/run.sh`.

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

## 2026-02-16 (Backlog: `--agent-id` Arg Order)

- Made the Rust CLI accept `--agent-id=<id>` in addition to `--agent-id <id>`, and strip it regardless of where it appears in argv: `crates/but-engineering-rewrite/src/main.rs`.
- Updated the harness stub to accept `--agent-id` anywhere (including `--agent-id=...`) and keep behavior aligned with Rust for observer commands (`agents`/`claims`): `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Added harness Case 74 to lock in the behavior for `check --path ... --agent-id B` and `--agent-id=B`: `crates/but-engineering-rewrite/harness/run.sh`.
- Fixed a few bash `set -u` footguns around empty arrays while shifting argv in the stub (so `claims` with no extra args doesn't crash).
- Strengthened the harness contract with Case 75 so `read`, `claims`, and `agents` also accept `--agent-id` placed after the subcommand (including `--agent-id=...`): `crates/but-engineering-rewrite/harness/run.sh`.
- No Rust/stub code changes were needed for Case 75; the harness stayed green, confirming behavior was already consistent (and now has tighter coverage).

## 2026-02-16 (Scored Eval: Reduce Redundant `read`)

- Tried removing `check.action_plan`'s leading `read` to reduce scored-run overhead, but harness Case 41 requires it for conflict repair guidance; reverted to keep harness behavior stable.
- Updated the scored rewrite compare prompt to run `read` once up front and to skip redundant reads unless `action_plan` explicitly calls for it: `crates/but-engineering-compare/eval/promptfooconfig.compare-rewrite.fast.yaml`, `crates/but-engineering-compare/eval/promptfooconfig.compare-rewrite.yaml`.
- Why: the prompt previously asked for `read` before every `check`, and `check` also suggests `read` on conflicts, causing duplicated coordination commands and worse overhead ratio.

## 2026-02-16 (E2E: Strengthen Drift Prompt Noise)

- Strengthened the Drift v1 agent prompt with extra “wrong suggestions” (e.g. `which`, `--help`, “run `read` first”) to better simulate longer-lived distraction without changing the required command sequence: `crates/but-engineering-rewrite/e2e/prompts/drift.step1.codex.txt`.
- Strengthened the Drift v2 agent prompt with more explicit trap instructions (don’t claim the claimed path, don’t skip `digest`, don’t use `read` for file contents) and added a small “avoid tool thrash” guardrail list: `crates/but-engineering-rewrite/e2e/prompts/drift_v2.step1.codex.txt`.
- Why: keep drift scenarios meaningful as the agents get better; more realistic noise should reduce “prompt gaming” and push the model to rely on tool state.

## 2026-02-16 (Scored Check Output: Action Plan Hints)

- Added `action_plan_hints` to `check` JSON in the Rust CLI so scored runners can detect required coordination steps (`read/post/ack/retry-check/claim`) via booleans instead of brittle command-string parsing: `crates/but-engineering-rewrite/src/main.rs`.
- Kept Rust and harness stub in parity by adding the same `action_plan_hints` shape/derivation in the stub `check` output: `crates/but-engineering-rewrite/harness/bin/but-engineering-rewrite`.
- Why: this is a small slice of backlog item 3 (“make check output more actionable for scoring”) without changing existing `action_plan` behavior or breaking harness expectations.
- What failed: no scored compare run in this iteration; validation was limited to the fast harness gate.

## 2026-02-16 (Checkpoint: Backlog Re-baseline)

- Ran `./crates/but-engineering-rewrite/harness/run.sh` as the iteration gate; it exited 0 (green), so this checkpoint stays within the "keep harness green" top priority.
- Re-audited CI coverage and confirmed deterministic harness/E2E workflows already exist and are wired in GitHub Actions: `.github/workflows/test-but-engineering-rewrite-harness.yml`, `.github/workflows/test-but-engineering-rewrite-e2e.yml`.
- Updated the living backlog to mark deterministic CI integration as done and move the next immediate slice to replayable agent-mode traces: `crates/but-engineering-rewrite/docs/08-backlog.md`.
- Why: the backlog should reflect current reality so the next coding slice targets unmet work instead of re-implementing landed items.
- What failed: this checkpoint did not add product/harness behavior; it was a green-run + backlog alignment pass.

## 2026-02-16 (E2E Replay: Trace Artifact Paths)

- Added `stdout_file` and `stderr_file` fields to each E2E trace row so provider-run outputs can be inspected directly from `trace.jsonl` without re-running live agents: `crates/but-engineering-rewrite/e2e/run.sh`.
- Wired trace emission to persist those file paths from `run_with_timeout` for all CLI/agent steps while keeping existing replay checks unchanged and backward-compatible with older traces.
- Why: this is a small slice of the “replayable E2E traces for agent-mode runs” backlog item, focused on making captured runs easier to debug and audit.
- Validation: reran the fast harness gate (`./crates/but-engineering-rewrite/harness/run.sh`) and kept it green.
- What failed: this iteration did not add a new E2E scenario assertion for the new trace fields; it focused on capture/inspectability only.
