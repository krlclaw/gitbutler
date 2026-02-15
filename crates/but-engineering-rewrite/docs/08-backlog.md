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

All 5 scenarios passing in both `--no-agents` (deterministic) and `--provider codex` (real agent) modes.

## Immediate priorities (next 1–2 iterations)
1) **Keep the harness green and checkpoint**
   - Run: `./crates/but-engineering-rewrite/harness/run.sh` until EXIT 0.
   - Commit once with proof in the message (harness tail + git status).

2) **Add CI integration for E2E suite**
   - E2E tests are stable enough to run regularly
   - Consider: deterministic mode only in CI, record/replay for agent runs

3) **Strengthen drift scenario**
   - Add more distraction content to prompt (simulate longer-lived session)
   - Add turn-count assertion to prevent infinite loops
   - Test with Claude once tool-executing mode is wired

## Future / nice-to-have
- Record/replay for E2E traces (like harness replay, but for agent transcripts).
- Small "runner" that executes `action_plan` automatically (with guardrails).
- Multi-model testing (Codex + Claude mix in same coordination channel).
