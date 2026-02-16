# Rewrite Full Iteration Log

## Iteration A (prompt tuning: coordination message clarity)

### Changes
- crates/but-engineering-compare/eval/promptfooconfig.compare-rewrite.yaml
  - Added explicit coordination-message quality requirements to the rewrite full scenario prompt:
    - blocked/skipped posts must include file + blocker + next step + ETA/trigger
    - completed-file updates must include file + release status
    - final summary must enumerate each requested file with status + next step
- crates/but-engineering-compare/eval/providers/engineering-integration.ts
  - Strengthened appended policy prompt with concise message-quality constraints:
    - file-specific blocked/completed status + blocker reason + next step/ETA
    - final done summary must cover each requested file with status + next step

### Validation loop
1) npm run build ✅
2) rewrite full (promptfooconfig.compare-rewrite.yaml, SCORED only) ⚠️
   - output: output/scored-rewrite-full.json
   - latest run: eval-H12-2026-02-16T07:21:24
   - result: failed due provider/runtime error stream (codex_core::rollout::list: state db missing rollout path ...), score collapsed to 0
3) legacy/rewrite fast sanity checks ⚠️
   - attempted, but same runner/runtime instability prevented clean completion in this session

### Score notes
- Baseline target from request: Message Quality = 10/25 (rewrite full)
- Current session full-run artifact is infra-failed; no reliable post-change Message Quality measurement available.
- Latest failed artifact reports overall 0 because provider errored, not because rubric logic changed.
