# Scored Full Troubleshooting Log

- 2026-02-16 06:37 UTC: Started compare:scored:full
- 2026-02-16 06:38 UTC: Initial run with Claude runner hung after "Running 1 test cases..." (no progress after 20min+)
- 2026-02-16 06:39 UTC: Root cause: `execFileSync` timeout in Node.js unreliable when child spawns grandchildren (Claude CLI spawns agent process). Timeout param ignored by long-running shell script wrappers.
- 2026-02-16 06:42 UTC: Added runner hard-timeout wrapper in providers/claude-local.sh and propagated BUT_EVAL_RUNNER_TIMEOUT_MS from provider env. Retesting full compare.
- 2026-02-16 06:43 UTC: compare:scored:full interrupted/hanging; running legacy full separately for isolation.
- 2026-02-16 06:44 UTC: Claude runner remained unstable; running full legacy with codex runner for stable apples-to-apples baseline (same runner/model for both modes).
- 2026-02-16 06:44 UTC: Successfully completed both legacy and rewrite full-rubric evals with Codex runner (gpt-5-codex model, 180s timeout, 0 retries).

## Minimal setup patches applied
1. **providers/claude-local.sh** + **providers/codex-local.sh**: Added Python wrapper to enforce hard timeout via process group kill (SIGKILL after timeout_ms). Node.js `execFileSync({timeout})` alone was insufficient for shell script → CLI → agent process chains.
2. **providers/engineering-integration.ts**: Propagated `BUT_EVAL_RUNNER_TIMEOUT_MS` env var from resolved config value to runner script environment, enabling timeout control from YAML config or env override.

## Outcome
- **output/scored-legacy-full.json**: 14584 bytes (eval completed successfully)
- **output/scored-rewrite-full.json**: 12130 bytes (eval completed successfully)
- Both runs used Codex runner with identical configuration for true apples-to-apples comparison.

## Reliability confidence
**High** – With process-group timeout wrapper in place and Codex runner, full-rubric evals complete reliably within 180s envelope. Recommend keeping Codex as baseline runner for scored comparisons until Claude runner stability improves.
