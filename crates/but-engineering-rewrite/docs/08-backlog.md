# Backlog / Next work (but-engineering-rewrite)

This is the *living* backlog that the watchdog + coding agents should use to pick the next slice.

## Guiding goal (don't lose the plot)
Build primitives + UX outputs that produce **net-positive coordination** between real coding agents (Codex + Claude) while keeping the system fast and non-blocking by default.

Progress only counts when either:
- the fast harness adds realism (new/stronger scenario) **and is green**, or
- an E2E (real-agent) test is added/improved and becomes stable enough to run regularly.

## Status: E2E Scenarios
✅ **smoke** - basic post/read/eval cycle
✅ **collision** - claim conflict → read/ack → release → proceed
✅ **discovery** - high-signal discovery → brief → action
✅ **triangle** - 3-agent dependency hints (A→B, noise from C)
✅ **drift** - runtime nonce anti-gaming (agent must discover via tools, not prompt)
✅ **drift_v2** - distraction + mid-task re-orientation (B must use eval+brief+digest, avoid claimed path, claim unclaimed, and post ack plan)

All 6 scenarios passing in both `--no-agents` (deterministic) and `--provider codex` (real agent) modes.

## Immediate priorities (next 1–2 iterations)
1) **Keep the harness green and checkpoint**
   - Run: `./crates/but-engineering-rewrite/harness/run.sh` until EXIT 0.
   - Commit once with proof in the message (harness tail + git status).

2) ✅ **CI integration for deterministic harness + E2E is in place**
   - Workflows: `.github/workflows/test-but-engineering-rewrite-harness.yml` and `.github/workflows/test-but-engineering-rewrite-e2e.yml`
   - Current policy: deterministic (`--no-agents`) in CI; real-agent runs remain local/manual.

3) **Next slice: replayable E2E traces for agent-mode runs**
   - Keep deterministic CI as the hard gate.
   - Add record/replay hooks for provider runs so regressions are inspectable without re-running live agents.

## Future / nice-to-have
- Record/replay for E2E traces (like harness replay, but for agent transcripts).
- Small "runner" that executes `action_plan` automatically (with guardrails).
- Multi-model testing (Codex + Claude mix in same coordination channel).

## Compare-driven loop (but-engineering-compare)

### Active benchmark: scored multi-file mixed ownership
Owner: Codex
Goal: rewrite score >= 0.9 consistently and beat legacy.

Next slices:
1) **Fix rewrite CLI robustness: accept --agent-id anywhere**
   - Symptom: compare fixture passes `--agent-id` after subcommand; rewrite returned internal_error.
   - Done when: `but-engineering-rewrite check/read/post/claim` work regardless of arg order.

2) **Stop agents from “tool thrash” in scored eval**
   - Add explicit instruction in compare prompt to avoid `which/--help/strings/netstat/strace`.
   - If still thrashing: improve Skill(but-engineering-rewrite) to be more directive (minimal, file-scoped).

3) **Make check output more actionable for scoring**
   - Ensure `check` returns clear decision + action_plan that includes required ack/discovery steps.
   - Improve messages/discovery signals so the scoring rubric can detect correct behavior.

4) **Add a repeat runner for scored eval**
   - `pnpm run scored:rewrite:fast --repeat N` (or a wrapper script) + aggregate mean/variance.
   - Goal: reduce flakiness + prevent regressions.
