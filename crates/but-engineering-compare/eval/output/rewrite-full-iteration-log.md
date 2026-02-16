# Rewrite Full Rubric Optimization Log

## Baseline (Iteration 0) - b4bbb8c3d

**Score: 60/100**

Breakdown:
- Turn Efficiency: 20/20 ✓
- Message Quality: 15/25 (-10)
- State Check Completeness: 20/20 ✓
- Overhead Ratio: 0/15 (-15) ← **PRIMARY BLOCKER**
- Edge Cases: 0/10 (-10)
- Safety: 10/10 ✓

**Analysis:**
- Coordination overhead is 77.4% (24/31 commands)
- Target: 30-45% ratio for full 15 points
- Too many redundant `read` operations (4 reads when 1 initial + selective is enough)
- Need to eliminate unnecessary coordination round-trips

**Next Fix:** Reduce redundant `read` commands in agent behavior.

## Iteration 1 - improve message extraction for shell-escaped quotes

**Change:** Make `extractMessageFromCommand` handle zsh-wrapped commands where the post text is passed as `\"...\"`.

**Score: 60 → 75 (+15)**

Observed improvements:
- Message extraction now captures the peer-a block message (auth.rs), increasing file-specific actionable messages.
- Safety + state completeness remain at full marks.

Next low-hanging fruit:
- Edge cases dimension still 0/10 (need explicit db.rs/config.rs claim/renew/stale messaging).
- Overhead ratio dimension still likely low.

## Iteration 1 - prompt policy hardening for edge-case communication

Changes:
- Updated  in  to explicitly require:
  - one concise file-specific skip/escalation post when check reports blocked/deny/risky/unowned/contested/stale
  - explicit claimed/renewed notes for claim/renew actions
  - concrete post content and complete per-file final summary

Validation:
1)  ✅
2) rewrite full rubric ⚠️ unstable in this run (scored output returned 0 due assertion receiving non-JSON/noisy provider output)
3) legacy fast sanity ✅ ( generated, eval passed)

Notes:
- Current run hit runner-output noise (codex rollout stderr) before JSON payload; scored assertion treated this as 0.
- Follow-up should harden provider-output extraction or assertion parser to recover embedded JSON before rescoring.

## Iteration 1 - prompt policy hardening for edge-case communication

Changes:
- Updated buildPolicyPrompt() in providers/engineering-integration.ts to explicitly require:
  - one concise file-specific skip/escalation post when check reports blocked/deny/risky/unowned/contested/stale
  - explicit claimed/renewed notes for claim/renew actions
  - concrete post content and complete per-file final summary

Validation:
1) npm run build - PASS
2) rewrite full rubric - UNSTABLE in this run (scored output returned 0 due assertion receiving non-JSON/noisy provider output)
3) legacy fast sanity - PASS (scored-legacy-fast.json generated, eval passed)

Notes:
- Current run hit runner-output noise (codex rollout stderr) before JSON payload; scored assertion treated this as 0.
- Follow-up should harden provider-output extraction or assertion parser to recover embedded JSON before rescoring.
