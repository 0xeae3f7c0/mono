#!/usr/bin/env bash
# Merges open dependency PRs (label kind/deps) that have waited longer than
# MAX_AGE_HOURS and have no failing checks. Checks that are still pending are
# bypassed with --admin, so OWNER_TOKEN must belong to a repository admin.
# Major updates (deps/major) and PRs with failing checks are left for a human;
# the latter get status/blocked.
#
# Usage: MAX_AGE_HOURS=3 DRY_RUN=false GH_TOKEN=... scripts/merge-stale-deps.sh
set -euo pipefail

MAX_AGE_HOURS=${MAX_AGE_HOURS:-3}
DRY_RUN=${DRY_RUN:-false}
cutoff=$(date -u -d "-$MAX_AGE_HOURS hours" +%s)
merged=0

rows=$(gh pr list --state open --label kind/deps --limit 200 \
  --json number,createdAt,labels,statusCheckRollup \
  --jq '.[] | "\(.number) \(.createdAt) \([.labels[].name] | join(",")) \([.statusCheckRollup[]? | select(.conclusion == "FAILURE" or .conclusion == "TIMED_OUT" or .conclusion == "CANCELLED")] | length)"')

while read -r number created labels failed; do
  [[ -n "$number" ]] || continue
  (( $(date -u -d "$created" +%s) <= cutoff )) || continue
  if [[ ",$labels," == *",deps/major,"* ]]; then
    echo "#$number: major update, left for review"
    continue
  fi
  if [[ "$failed" -gt 0 ]]; then
    echo "#$number: $failed failing check(s), marking blocked"
    [[ "$DRY_RUN" == true ]] || gh pr edit "$number" --add-label status/blocked || true
    continue
  fi
  echo "#$number: open since $created, merging"
  [[ "$DRY_RUN" == true ]] && continue
  if gh pr merge "$number" --rebase --delete-branch --admin || gh pr merge "$number" --rebase --delete-branch; then
    merged=$((merged + 1))
  else
    echo "::warning::Could not merge #$number (likely a conflict); the next update run will refresh it."
  fi
done <<<"$rows"

echo "Merged $merged pull request(s) older than $MAX_AGE_HOURS hour(s)."
