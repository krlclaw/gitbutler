# but-engineering (rewrite) — Spec (v1)

This is the spec for a coordination tool for multiple coding agents working in the **same repository**.

Kiril note (2026‑02‑14): **We do not need to match the previous API/workflow.** The only requirement is that it’s genuinely useful for agent coordination.

## 0) What success looks like

- Multiple agents can work concurrently without stepping on each other’s toes.
- Conflicts are prevented *before* edits happen (or surfaced early with a clear resolution path).
- Agents reliably “remember” to coordinate, even across long sessions.
- It’s easy to integrate into:
  - Claude Code (hooks)
  - Codex / other agents (wrapper flow)
  - humans (optional TUI / CLI)

## 1) Design principles

1) **Git is the source of truth for code**, not the coordination system.
2) Coordination state is **ephemeral** and optimized for liveness.
3) Prefer **deterministic enforcement at danger points** (pre-edit) over persuasion.
4) Keep agent instructions **tiny + identity framed**; prefer “who you are” over rule lists.
5) Every block must have a **clear exit condition**.
6) Provide **machine-actionable outputs** (reason codes + next steps) so wrappers can behave predictably.

## 2) Core concepts

### 2.1 Agents
An agent is identified by `agent_id` (string). Optional fields:
- `status`: what I’m doing now
- `plan`: what I’m about to do next (short)

### 2.2 Channel
A flat message stream (Slack-like):
- message types: `message`, `block`, `discovery`
- supports mentions: `@agent-id`

### 2.3 Intents (advisory) / leases
Agents can register **advisory intents** on files/paths to surface collisions early.

**Key properties**
- Intents are **leases** with TTL (default e.g. 15m) and refresh-on-use.
- By default intents are **not hard locks**: they are signals for collision detection and coordination.
- Escape hatch: callers can request strict enforcement at check time (see `check --strict`).

### 2.4 “Danger points” enforcement
- **pre-edit** is the critical gate. If the runtime supports it (Claude hooks), block edits.
- If runtime doesn’t support hooks (Codex), provide a wrapper `check` flow.

## 3) Minimal product surface (v0)

We intentionally keep the surface minimal. Everything below is designed to be scriptable.

### 3.1 `post`
Post a message.

### 3.2 `read`
Read messages, optionally block-wait.

### 3.3 `status` / `plan`
Set/clear status and plan.

### 3.4 `agents`
List active agents.

### 3.5 `claim` / `release` / `claims`
Manage leases.

### 3.6 `check`
A machine-actionable “should I edit this?” decision.

### 3.7 `done`
Post completion summary + cleanup (release claims, clear plan/status).

### 3.8 Optional: `lurk`
Human-only TUI viewer.

## 4) Contracts (JSON)

All commands output JSON except `lurk` (interactive) and `eval user-prompt-submit` (plain text).

### 4.1 Check API (canonical contract)

`check` returns:
- `decision`: `allow | deny | warn`
- `reason_code`: stable string enum
- `blocking_agents`: list
- `action_plan`: ordered next steps (commands)
- `strict`: optional boolean input (default `false`). If `true`, `check` MAY return `deny` when another agent has an active intent.

Reason codes (initial):
- `claimed_by_other`
- `recent_activity_on_file`
- `identity_missing`
- `no_conflict`

Action kinds (initial):
- `read_channel`
- `post_intent`
- `claim_file`
- `wait_for_release`
- `retry`
- `proceed`

## 5) Hook semantics (Claude Code)

### 5.1 `eval user-prompt-submit`
Output **short repeated nudge + live novel state**.

Invariant 1-liner (repeated):
- “Announce what you’ll do (files), read the channel, then proceed.”

Novel state:
- active agents
- unread messages preview
- current claims summary

### 5.2 `eval pre-tool-use`
If an edit targets a file with an active intent by another agent:
- default behavior: return `warn` with actionable next steps (coordination without hard locking).
- strict behavior (opt-in): return `deny` with a satisfiable reason.
- auto-post a short message when strict-denying indicating who is blocked and why.

If the file was recently mentioned but not claimed:
- return `warn` advisory context and encourage claim.

## 6) Testing strategy (critical)

We will build a disposable repo harness that can run:

1) **Two-agent conflict test**
- Agent A claims file X.
- Agent B attempts to edit X.
- Expected: default `check` returns `warn` (advisory) with action_plan.
- Strict variant: `check --strict` returns `deny` with action_plan.

2) **Lease expiry test**
- A claims X with TTL.
- TTL expires.
- B can acquire claim after backoff.

3) **Habit formation test** (hook relevance)
- Simulate multiple turns; ensure `user-prompt-submit` remains short but changes due to live state.

4) **Deadlock avoidance**
- Ensure deny responses have explicit next steps and stop conditions.

## 7) Non-goals (v0)

- Distributed multi-machine coordination (we can add later).
- Perfect semantic dependency detection.
- Long-term audit/provenance beyond what Git already provides.
