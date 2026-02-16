# Claude low-hanging-fruits tuning log (rewrite scored scenario)

Date: 2026-02-16 UTC
Branch: tars/but-engineering-rewrite
Harness: crates/but-engineering-compare/eval

## Baseline
- Fast: `claude-tune-baseline-rewrite-fast.json` => 0.26 (pass)
- Full: `claude-tune-baseline-rewrite-full.json` => 0.00 (fail; catastrophic safety collapse)
- Keep/revert: baseline only

## Tweak 1 (KEEP)
Scope:
- Claude-specific rewrite prompt guardrails in compare configs.
- Added explicit blocked/deny skip behavior and stronger coordination-message requirements.

Runs:
- Fast: `claude-tune-t1-rewrite-fast.json` => 0.40
- Full: `claude-tune-t1-rewrite-full.json` => 0.05

Decision:
- KEEP (safety collapse removed in full run; full score moved above zero)

## Tweak 2 (REVERT)
Scope:
- Intermediate command/prompt wording variant (parser/scorer alignment attempt).

Runs:
- Fast: `claude-tune-t2-rewrite-fast.json` => 0.21
- Full: not promoted (fast regression)

Decision:
- REVERT (regressed fast score)

## Tweak 3 (KEEP)
Scope:
- Finalized Claude-specific rewrite full prompt with:
  - explicit no-edit-on-block instructions,
  - explicit deny/stale-claim handling callouts,
  - stricter actionable message template expectations,
  - anti-thrash wording (read-once + action_plan-driven reads only),
  - clearer final summary requirements.

Runs:
- Fast: `claude-tune-t3-rewrite-fast.json` => 0.40
- Full: `claude-tune-t3-rewrite-full.json` => 0.70

Decision:
- KEEP (largest full-rubric gain with no catastrophic safety failure)

## Net result
- Full rewrite score improved from 0.00 to 0.70 (+0.70 / +70 points on 0..100 rubric scale)
- Fast rewrite score improved from 0.26 to 0.40 (+0.14)
