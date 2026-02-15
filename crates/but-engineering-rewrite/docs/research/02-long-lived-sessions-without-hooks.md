# Research: Long-lived agent sessions without hooks (tool-use drift + anti-gaming E2E)

## Problem framing
In longer-lived agent sessions (e.g. Claude Code in a repo for hours), agents can *drift*:
- stop using the repo-specific coordination tool
- forget the local “ritual” (read channel → check claims → ack → proceed)
- regress into “just edit files” mode even when coordination state exists

Hooks help (re-inject instructions per user prompt), but we want to get as far as possible **without** them.

## Observed failure modes
1) **Tool-use amnesia / drift**
   - Tool was used earlier, but later turns stop calling it.
   - Often happens after distractions: tests failing, unrelated questions, context window growth.

2) **Protocol bypass**
   - Agent executes shell/git edits directly without checking coordination state.

3) **Brittle I/O assumptions**
   - Tests and agents expect “stdout is single JSON” and die on noise.

4) **Prompt drift as an operational risk**
   - Even without code changes, behavior shifts due to model updates, context accumulation, or subtle changes.

## Useful external references (high signal)

### Agent testing pyramid (deterministic → replay → probabilistic)
Block Eng argues agent testing layers should be organized by tolerated uncertainty:
- deterministic foundations
- reproducible reality via **record/replay**
- probabilistic benchmarks (success rate)
- “vibes/judgment” with rubrics

Source: https://engineering.block.xyz/blog/testing-pyramid-for-ai-agents
Key takeaway for us: **don’t run live LLM tests in CI**; CI validates deterministic + replayed fixtures.

### Coordination patterns (Agent Mail / advisory reservations)
Summaries of “Agent Mail” + advisory file reservations emphasize:
- persistent inbox/digest
- advisory leases
- human-auditable state

Source: https://www.bryanwhiting.com/ai/agent-mail-beads-coordinated-ai-coding-agents-how/
(We’re implementing a smaller, CLI-first subset.)

### Prompt drift concept
“Prompt drift” = gradual misalignment over time due to model updates, context accumulation, tool inconsistencies.
Source: https://www.comet.com/site/blog/prompt-drift/
Key takeaway: drift is expected; mitigate via observability + offline evals.

## Design implications for but-engineering-rewrite
To reduce “forgetting the tool” without hooks, the tool must create a *self-reinforcing loop*:

1) **Make the correct first step obvious**
   - Provide a cheap, memorable “orientation” command (e.g. `eval user-prompt-submit`).
   - This should output a short ritual: "announce what you’ll do; check claims; read messages; then proceed".

2) **Make CLI outputs self-instructing**
   - `check` must return an actionable `action_plan` that naturally includes “read” and “ack once” steps.
   - `brief/digest` should include “next_steps” with runnable commands.

3) **Persist state + hints in a place agents naturally query**
   - Agents are more likely to run `brief`/`digest` than read docs.
   - Make those surfaces contain enough to re-orient after drift.

4) **Anti-gaming evaluation**
   - A test that can be passed by printing a sentinel is worthless.
   - E2E should require the agent to *discover and use* a runtime-generated nonce from tool state.

## Proposed high-ROI E2E test (anti-gaming): “re-orientation + drift resistance”

### Core idea
- The E2E runner generates a fresh random nonce each run.
- The nonce is placed into coordination state via a discovery/message (not in the agent prompt).
- The agent must:
  1) run the orientation command (`./but-engineering-rewrite eval user-prompt-submit`)
  2) use the tool surfaces (`brief` or `read`) to find the nonce
  3) execute a suggested next_step that posts the nonce back

Pass criteria: the nonce appears in agent B’s posted message in the DB.

Why it’s hard to game:
- nonce changes every run
- the prompt does not contain the nonce
- the only way to obtain it is to use the coordination tool correctly

### Optional drift simulation
Within the prompt, add large “distraction” text and unrelated sub-tasks before/after.
Then require the agent to re-orient again (run `brief`/`check`) before finishing.

### What we assert
- trace.jsonl shows B invoked the tool at least N times (eval + brief/read + post)
- B’s message contains the nonce
- B didn’t spam (only one ack)

