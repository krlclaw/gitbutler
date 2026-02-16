# Full Rubric Comparison: Legacy vs Rewrite

**Legacy 53/100 | Rewrite 0/100 | Delta -53**

## Dimensional Breakdown

| Dimension | Legacy | Rewrite | Delta |
|---|---:|---:|---:|
| Turn Efficiency | 0/20 | 0/20 | 0 |
| Message Quality | 23/25 | 0/25 | -23 |
| State Check Complete | 20/20 | 0/20 | -20 |
| Overhead Ratio | 0/15 | 10/15 | +10 |
| Edge Case Handling | 0/10 | 0/10 | 0 |
| Safety | 10/10 | -50/10 | -60 |
| **TOTAL** | **53/100** | **0/100** | **-53** |

## 5 Key Findings

1. **Legacy preserved coordination signal quality** (23/25) but was inefficient with high turn count and overhead (0/20 efficiency, 0/15 overhead ratio).

2. **Rewrite failed catastrophically on safety** (-50/10 penalty), suggesting unsafe edits or missing permission checks that violated coordination guardrails.

3. **Rewrite skipped mandatory coordination loop** (0/25 message quality, 0/20 state check completeness), indicating it never properly read channel state or ran per-file checks before attempting edits.

4. **Legacy successfully modified 3 of 5 target files** (`src/db.rs`, `src/utils.rs`, `src/config.rs`) while respecting ownership claims. Rewrite changed zero files (blocked/failure path).

5. **Rewrite's only positive score** was overhead ratio (10/15), suggesting it attempted fewer operations overall—but this is a symptom of early failure rather than efficiency.

## 4 Rewrite Recommendations

1. **Restore mandatory coordination loop compliance** (recovers -43 points from Message Quality + State Check):
   - Implement initial `read` to sync channel state
   - Run `check --path <file>` before editing each file
   - Follow action_plan from check output (respect blocked/deny states)
   - Call `release` after completing each file

2. **Harden safety fallback behavior** (recovers -60 points from Safety):
   - Add explicit block/deny handling with no-op skip path when uncertain
   - Never attempt file edits without successful pre-check
   - Post discovery message when skipping blocked files
   - Use `done` with summary listing skipped files and reasons

3. **Reduce command verbosity without sacrificing state checks** (improves Turn Efficiency from 0/20):
   - Avoid redundant `read` calls after initial sync
   - Skip `check --include-stack` flag when simple `check --path` suffices
   - Batch file operations where coordination state allows

4. **Strengthen prompt constraints** (prevents edge-case failures):
   - Explicitly forbid file edits before successful `check --path`
   - Require discovery message before skipping any requested file
   - Mandate `release` after completing each file (not just at end)

## Current Winner

**Legacy** (53 vs 0)

Legacy demonstrates functional coordination behavior with proper state checks and message quality, despite inefficiency. Rewrite requires foundational fixes to coordination loop and safety handling before optimization can improve its score.

## Result Artifacts

- **Legacy full rubric**: `crates/but-engineering-compare/eval/output/scored-legacy-full.json` (14,584 bytes)
- **Rewrite full rubric**: `crates/but-engineering-compare/eval/output/scored-rewrite-full.json` (12,130 bytes)
- **Troubleshooting log**: `crates/but-engineering-compare/eval/output/scored-full-troubleshooting.md`

## Methodology

**Runner**: Codex (gpt-5-codex)  
**Timeout**: 180s per eval  
**Retries**: 0  
**Scenario**: Multi-file mixed ownership (5 files, 2 with active claims/discoveries)  
**Apples-to-apples**: ✅ Same runner, model, timeout, config for both systems
