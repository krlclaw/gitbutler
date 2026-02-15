# Case 43: Unread Updates (Avoid Path Substring False Positives)

## Goal
Reduce coordination noise by preventing `check --path` from treating substring path overlaps as relevant updates.

## Why This Matters
- Humans often mention nearby files (backups, temp files, generated variants) that share a prefix with the real target.
- Naive substring matching creates false-positive "unread relevant updates" and unnecessary `@X: ack:` suggestions.
- Better relevance yields more trustworthy coordination signals.

## Scenario
1. Agent A posts a coordination note about `src/app.txt.bak` (not the real `src/app.txt`).
2. Agent B runs `check --path src/app.txt` and should *not* see A's note as an unread relevant update.
3. Agent A posts a second note that explicitly mentions `src/app.txt`.
4. Agent B runs `check --path src/app.txt` and should see the unread update and a suggested `@A: ack: ...` closure step.

