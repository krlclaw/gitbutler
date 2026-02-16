# Claude skill tuning log (but-engineering-rewrite)

Date: 2026-02-16 (UTC)
Branch: `tars/but-engineering-rewrite`
Scope: minimal edits to `crates/but-engineering-compare/eval/skills/but-engineering-rewrite.SKILL.md` only.

## 0) Baseline reproduction (serialized Claude)

Important harness note while reproducing:
- The fixture defaults `BUT_EVAL_MODE` to `legacy`. If you run `promptfoo eval` manually (not via npm scripts), you **must** set `BUT_EVAL_MODE=rewrite` or you will accidentally install the legacy skill and see legacy CLI commands.

### Baseline outputs (Claude)

- Rewrite FAST (manual run, missing `BUT_EVAL_MODE=rewrite`): `output/claude-tune-baseline-rewrite-fast.json`
  - `success=true`, `score=0.26`
  - Observed failure pattern: mixed/partial progress, max-turns behavior.

- Rewrite FULL (manual run, missing `BUT_EVAL_MODE=rewrite`): `output/claude-tune-baseline-rewrite-full.json`
  - `success=false`, `score=0`
  - `resultMeta.subtype=error_max_turns`
  - **Top failure patterns:**
    - Uses the wrong CLI: `but-engineering ...` (legacy) instead of `but-engineering-rewrite ...`
    - Adds legacy-only flags: `--include-stack`
    - Uses legacy positional/shape variants (e.g. `check src/auth.rs`) instead of `check --path ...`
    - Coordination messages degraded (bodies came through as `undefined` in provider capture)

These patterns cause the rewrite-mode coordination contract to break, leading to assertion failure and/or turn thrash.

## 1) Tweak T1 (WINNER): harden skill against legacy CLI/flags + provide copy-paste templates

### Change
Edited `crates/but-engineering-compare/eval/skills/but-engineering-rewrite.SKILL.md`:

- Added explicit constraint:
  - **"Use only the `but-engineering-rewrite` CLI for coordination in this task. Never call `but-engineering`."**
- Added explicit constraint:
  - **"Do NOT add legacy flags not supported by rewrite (`--include-stack`, positional-path variants)."**
- Added **Command Templates (copy exactly)** block for the rewrite CLI shapes.

### Why this helps Claude
Claude was "helpfully" generalizing from older evals/docs and calling `but-engineering` with `--include-stack`.
These additions:
- Remove ambiguity about which CLI is authoritative.
- Prevent the most common failure mode (legacy flags / legacy command shapes).
- Provide a low-friction, copy-exact command skeleton that reduces tool thrash.

### Post-tweak results (with correct env)

Run with: `BUT_EVAL_MODE=rewrite`

- Rewrite FAST (Claude): `output/claude-tune-t3-rewrite-fast.json`
  - `success=true`, `score=0.4`
  - CLI usage check:
    - `but-engineering-rewrite`: **15**
    - `but-engineering` (legacy): **0**
    - `--include-stack`: **0**

- Rewrite FULL (Claude): `output/claude-tune-t3-rewrite-full.json`
  - `success=true`, `score=0.7`
  - CLI usage check:
    - `but-engineering-rewrite`: **20**
    - `but-engineering` (legacy): **0**
    - `--include-stack`: **0**

### Codex sanity check (not catastrophically regressed)

- Rewrite FULL (Codex): `output/codex-tune-sanity-rewrite-full.json`
  - `success=true`, `score=0.9`

## 2) Notes / next hypotheses

- If further improvements are needed, the next likely lever is reducing redundant coordination posts/reads and ensuring Claude always follows the scenario's “read only when action_plan says” rule.
- However, the current tweak already fixes the most damaging class of failures (legacy CLI/flags), while keeping diffs minimal.
