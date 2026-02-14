# Case 21: Multi-Step Convergence (3-Agent Coordination)

## Goal
Exercise a short multi-step coordination sequence where 3 agents start in a conflicting state and converge to a reduced-conflict plan:

- **A and B collide** on the same path (advisory collision via overlapping claims).
- **C makes it worse** by holding a broader directory claim that creates a triangle conflict.
- **Convergence**: one agent narrows scope (release broad claim and re-claim a non-overlapping path).
- **Actionability**: the final `check` output includes concrete next actions per agent (A/B/C), not just the caller.
- **Noise control**: blocking agents and plans are deduped (no duplicate agent ids).

## Harness Steps
1. A: `claim --path src/app.txt --ttl 15m`
2. B: `claim --path src/app.txt --ttl 15m`
3. C: `claim --path src/ --ttl 15m` (broad claim)
4. B: `check --path src/app.txt` (initial: 2 blockers)
5. C: `release --path src/` (narrow)
6. C: `claim --path src/other.txt --ttl 15m` (non-overlapping)
7. B: `check --path src/app.txt` (final: <= 1 blocker)

## Expected
- Initial `check` shows 2 blocking agents.
- Final `check` shows 0-1 blocking agents (collisions reduced).
- Final `check` includes `action_plan_by_agent` with specific next steps for each agent `A`, `B`, and `C`.
