# Backlog / Next work (but-engineering-rewrite)

This is the *living* backlog that the watchdog + coding agents should use to pick the next slice.

## Guiding goal (don’t lose the plot)
Build primitives + UX outputs that produce **net-positive coordination** between real coding agents (Codex + Claude) while keeping the system fast and non-blocking by default.

Progress only counts when either:
- the fast harness adds realism (new/stronger scenario) **and is green**, or
- an E2E (real-agent) test is added/improved and becomes stable enough to run regularly.

## Immediate priorities (next 1–2 iterations)
1) **Keep the harness green and checkpoint**
   - Run: `./crates/but-engineering-rewrite/harness/run.sh` until EXIT 0.
   - Commit once with proof in the message (harness tail + git status).

2) **Harden the E2E “real agent” test runner (opt-in slow suite)**
   - Runner exists: `crates/but-engineering-rewrite/e2e/run.sh` (scenarios: `smoke`/`collision`/`discovery`/`triangle`).
   - Next: keep adding deterministic scenarios that stress coordination signal/noise, and keep agent-spawn paths pinned and verdictable (Codex-only until Claude has a tool-executing mode wired in).

## High-value real-agent E2E tests (start with 2–3)

### E2E-01: Two-agent collision → read/ack → release → proceed
**Why:** validates the full coordination loop with an actual agent following action_plan.

Acceptance:
- Agent B runs `check --path …` and receives an actionable plan.
- B executes: `read`, then posts an ack that satisfies our “closure / anti-ping-pong” rules.
- A posts an update and/or releases the claim.
- B re-checks and proceeds without noisy repeated pings.

### E2E-02: High-signal discovery propagation → action
**Why:** ensures “share valuable findings” works end-to-end with a real agent.

Acceptance:
- A posts a high-signal discovery with suggested action.
- B’s `brief`/`digest` surfaces it.
- B executes at least one suggested next_step and posts back evidence.

### E2E-03: 3-agent triangle + dependency chain (A→B→C)
**Why:** prevents 2-agent bias; tests dedupe + actionable brief.

Acceptance:
- Brief is actionable (clear steps per agent) and avoids false dependency hints.

## Design constraints for E2E
- E2E tests are allowed to be slower and flaky at first; we iterate.
- Prefer **deterministic** scaffolding:
  - pinned prompts
  - explicit commands to run
  - captured traces
  - minimal degrees of freedom
- If a test flakes, add better instrumentation before “guessing”.
- Tool should work with:
  - Codex only
  - Claude only
  - mix of Codex + Claude

## Future / nice-to-have
- Record/replay for E2E traces (like harness replay, but for agent transcripts).
- Small “runner” that executes `action_plan` automatically (with guardrails).
