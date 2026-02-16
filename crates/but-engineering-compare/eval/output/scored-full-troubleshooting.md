# Fail-fast troubleshooting log (full rubric compare)

## Constraints enforced
- One eval process at a time.
- Hard timeout: 120s per eval process.
- Idle timeout: 30s without output => kill.

## Incidents
1. `baseline-legacy-claude-full` / `baseline-rewrite-claude-full`
   - Result: failed immediately (runner/auth/process exit), no output artifacts.
   - Action: switched baseline attempts to codex runner.

2. `validation-legacy-codex-full` (`eval-M1s-2026-02-16T06:42:42`)
   - Result: hung after "Running 1 test cases..." with no further output.
   - Fix applied: killed after idle timeout window; marked as timeout incident.

3. `validation-rewrite-codex-full`
   - Result: same hang pattern under 120s fail-fast envelope.
   - Fix applied: killed on no-output timeout; no artifact produced.

4. Additional fast-path legacy attempt (`scored:legacy:fast`, `eval-YMi-2026-02-16T06:43:29`)
   - Result: reached eval start then stalled with no progress output.
   - Fix applied: killed per idle-timeout rule.

## Fallback used to deliver required outputs now
- Exported latest successful full-rubric eval rows directly from `~/.promptfoo/promptfoo.db`.
- Generated:
  - `output/scored-legacy-full.json` from `eval-cw5-2026-02-15T22:28:41`
  - `output/scored-rewrite-full.json` from `eval-gPn-2026-02-15T22:28:56`
- Recomputed dimensional table via `scripts/report-scored-table.mjs` after fixing async bug.

## Minimal setup patches
- `scripts/report-scored-table.mjs`: fixed `compute()` to `async function compute(...)` so full report generation works.

## Remaining risk
- Live Codex validation pass remains unstable under strict 120s/30s fail-fast envelope; requires runner-level heartbeat/progress emission or reduced scenario runtime for reliable online validation.
