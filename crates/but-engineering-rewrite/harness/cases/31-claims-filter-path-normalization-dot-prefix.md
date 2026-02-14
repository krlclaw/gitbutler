# Case 31: Claims Filter (Path Normalization: Leading ./)

## Goal
Wrappers should be able to ask "who is working on this path?" without worrying about
common path spellings like a leading `./`.

## Scenario
1. Agent A claims a directory (`src/`).
2. Agent C claims a specific file inside it (`src/app.txt`).
3. Agent B requests a filtered view using a leading dot prefix:
   - `claims --path-prefix ./src/app.txt`

## Expected Behavior
- Output includes the directory claim (`src`) because it overlaps the requested path.
- Output includes the specific file claim (`src/app.txt`).
- The leading `./` does not change results compared to `claims --path-prefix src/app.txt`.

