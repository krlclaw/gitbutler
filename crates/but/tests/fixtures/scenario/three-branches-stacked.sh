#!/usr/bin/env bash

set -eu -o pipefail

source "${BASH_SOURCE[0]%/*}/shared.sh"

### General Description

# Three branches stacked: auth → shared (base), api → shared (base).
# Each branch has one commit with a file inside.
# "shared" is the base, "auth" and "api" are stacked on top.
git-init-frozen
commit-file M
setup_target_to_match_main

git checkout -b shared
  commit-file shared-file

git checkout shared -b api
  commit-file api-file

# Create auth stacked on shared (not on api)
git checkout shared -b auth
  commit-file auth-file

create_workspace_commit_once auth api
