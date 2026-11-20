#!/usr/bin/env bash
# Sets git author/committer to the GitHub account that owns GH_TOKEN, using
# its noreply address so commits are attributed to that account.
set -euo pipefail

login=$(gh api user --jq .login)
id=$(gh api user --jq .id)
name=$(gh api user --jq '.name // .login')

git config user.name "$name"
git config user.email "${id}+${login}@users.noreply.github.com"
echo "Committing as $name <${id}+${login}@users.noreply.github.com>"
