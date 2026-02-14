# Case 14: Claims Filter (Path Prefix Overlap)

## Goal
Wrappers need a way to show only the relevant active claims for a file/directory the user is about to edit, without dumping the entire lease table.

## Scenario
1. Agent A claims a directory (e.g. `src/`).
2. Agent C claims a specific file inside that directory (e.g. `src/app.txt`).
3. Agent B claims an unrelated file (e.g. `notes.txt`).
4. Another agent requests a filtered view:
   - `claims --path-prefix src/app.txt`

## Expected Behavior
- Output includes the directory claim (`src`) because it overlaps the requested path.
- Output includes the specific file claim (`src/app.txt`).
- Output does not include unrelated claims (like `notes.txt`).

## Notes
This is intentionally string-based and mirrors the overlap logic used by `check`:
exact match OR ancestor/descendant match on path segment boundaries.

