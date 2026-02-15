# Case 51: Quoted Then Resolve Is Closure (No Ack Needed)

## Goal
Avoid false non-closure: if a relevant unread update both *quotes* an `@<me>: resolve:` snippet and then includes a real directed `@<me>: resolve:` on a later line, `check --path` must treat the real one as explicit closure (no `@X: ack:` suggestion).

## Why This Matters
- People often quote prior messages (including resolves) before writing their actual reply.
- A naive "if anything before the match looks like quoting, ignore all resolves" heuristic can miss the real closure line and generate unnecessary acks.
- The useful behavior is: detect `resolve:` as closure on the specific line where it appears, not based on unrelated quoting on earlier lines.

## Scenario
1. Agent A posts a coordination message mentioning `src/app.txt` that includes:
   - a quoted `> @B: resolve: ...` line, then
   - a real `@B: resolve: ...` line.
2. Agent B runs `check --path src/app.txt`.
3. `check` surfaces A's message as an unread relevant update but does not suggest `@A: ack: ...`.

