# Case 24: Release Path Normalization (Leading ./)

## Goal
Ensure `release` normalizes common path spellings the same way `claim`/`check` do.

This avoids a coordination footgun where an agent claims `src/app.txt` but later
tries to unblock others with `release --path ./src/app.txt` and accidentally
does nothing.

## Harness Steps
1. A: `claim --path src/app.txt --ttl 15m`
2. A: `release --path ./src/app.txt` (leading `./`)
3. B: `check --path src/app.txt`

## Expected
- Release succeeds.
- B sees `allow` + `no_conflict` for `src/app.txt`.

