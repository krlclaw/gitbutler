# Research: Worktrees vs same-worktree coordination (high-signal)

Worktrees are the canonical approach for running multiple coding agents because they provide structural isolation (separate working dir + HEAD/index).

However, common reasons to *avoid* worktrees include:
- operational overhead (scripts, naming, cleanup)
- IDE/workspace friction and config drift
- cache/build duplication and artifact interference
- cognitive overhead (“where is the thing?”)
- coordination problems persist (discoveries/dependencies still conflict)

Implication: if we coordinate inside one worktree, the tool must provide:
- advisory collision detection (warn by default)
- high-signal discovery propagation
- dependency hints (provider/consumer)
- a shared, inspectable state/digest

Starter refs:
- https://devcenter.upsun.com/posts/git-worktrees-for-parallel-ai-coding-agents/
- https://dev.to/arifszn/git-worktrees-the-power-behind-cursors-parallel-agents-19j1
- https://www.bryanwhiting.com/ai/agent-mail-beads-coordinated-ai-coding-agents-how/
