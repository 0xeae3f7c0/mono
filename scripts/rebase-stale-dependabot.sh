#!/usr/bin/env bash
# repository tooling (scripts/)
# Comments "@dependabot rebase" on open Dependabot pull requests whose checks
# failed while their branch is behind the base branch: the fix may already be
# on main (for example a corrected lockfile), and Dependabot only rebases on
# its own when a PR has merge conflicts.
#
# At most one rebase request per PR every 24 hours, since each rebase reruns
# all of CI. Passing PRs are left alone (approve-ready-prs merges them), and
# "@dependabot recreate" is never sent: it discards commits pushed by hand.
#
# Dependabot ignores commands from bot accounts, so GH_TOKEN must belong to a
# user with write access (OWNER_TOKEN).
#
#   GH_TOKEN=... GITHUB_REPOSITORY=owner/repo [DRY_RUN=true] scripts/rebase-stale-dependabot.sh
set -euo pipefail

: "${GH_TOKEN:?GH_TOKEN is required}" "${GITHUB_REPOSITORY:?}"
repo=$GITHUB_REPOSITORY
dry_run=${DRY_RUN:-false}
cooldown=$((24 * 3600))
now=$(date +%s)

prs=$(gh pr list --repo "$repo" --state open --limit 1000 --app dependabot \
  --json number,isDraft --jq '.[] | select(.isDraft | not) | .number')

for pr in $prs; do
  info=$(gh pr view "$pr" --repo "$repo" --json baseRefName,headRefName,statusCheckRollup)

  failed=$(jq '[.statusCheckRollup[] | select((.conclusion // .state) as $c | ["FAILURE","ERROR","TIMED_OUT","STARTUP_FAILURE"] | index($c))] | length' <<<"$info")
  pending=$(jq '[.statusCheckRollup[] | select((.status // "COMPLETED") != "COMPLETED" or (.state // "") == "PENDING")] | length' <<<"$info")
  if (( failed == 0 || pending > 0 )); then
    continue
  fi

  base=$(jq -r .baseRefName <<<"$info")
  head=$(jq -r .headRefName <<<"$info")
  behind=$(gh api "repos/$repo/compare/$base...$head" --jq .behind_by)
  if (( behind == 0 )); then
    echo "#$pr: failing but up to date with $base, skipping"
    continue
  fi

  last=$(gh api --paginate "repos/$repo/issues/$pr/comments" \
    --jq '.[] | select(.body | test("^@dependabot (rebase|recreate)")) | .created_at' | tail -n 1)
  if [[ -n $last ]] && (( now - $(date -d "$last" +%s) < cooldown )); then
    echo "#$pr: rebase already requested at $last, skipping"
    continue
  fi

  echo "#$pr: $failed failed checks, $behind commits behind $base; requesting rebase"
  [[ $dry_run == true ]] || gh pr comment "$pr" --repo "$repo" --body "@dependabot rebase"
  sleep 1
done
