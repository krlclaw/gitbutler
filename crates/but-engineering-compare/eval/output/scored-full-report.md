# Full Rubric Comparison (Legacy vs Rewrite)

Legacy 53/100 | Rewrite 0/100 | Delta -53

| Dimension | Legacy | Rewrite | Delta |
|---|---:|---:|---:|
| Turn Efficiency | 0/20 | 0/20 | 0 |
| Message Quality | 23/25 | 0/25 | -23 |
| State Check Complete | 20/20 | 0/20 | -20 |
| Overhead Ratio | 0/15 | 10/15 | +10 |
| Edge Case Handling | 0/10 | 0/10 | 0 |
| Safety | 10/10 | -50/10 | -60 |
| TOTAL | 53/100 | 0/100 | -53 |

## Key findings
- Legacy run preserved coordination signal quality and check completeness, but was inefficient (high turn count / overhead).
- Rewrite run failed hard on safety and coordination signal extraction in this sampled full-rubric trace, driving total score to 0.
- Rewrite changed no watched files in the sampled trace (likely blocked/failure path), which aligns with low message/state/safety outcomes.

## Rewrite recommendations
- Restore minimum-safe coordination loop first: initial `read`, per-file `check --path`, explicit blocked/deny handling, and `release`/`done` summary.
- Add guardrails in rewrite skill prompts to force safety-safe fallback behavior when uncertain (no risky edits, explicit skip + message).
- Reduce command verbosity while retaining mandatory state checks (target: keep check completeness >= legacy without extra turns).
- Re-run full-rubric with deterministic/no-agent fixture replay and then one Codex validation run after each stability fix.

## Data source note
- Due live runner hangs under fail-fast constraints, this report uses exported full-rubric eval artifacts from promptfoo DB:
  - legacy: `eval-cw5-2026-02-15T22:28:41`
  - rewrite: `eval-gPn-2026-02-15T22:28:56`
