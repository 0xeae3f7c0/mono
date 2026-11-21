#!/usr/bin/env bash
# repository tooling (scripts/)
# Approves and merges open pull requests from bot accounts (Dependabot,
# github-actions) once they are ready: not a draft, no merge conflicts and every
# check finished without failing. Majors included.
#
# Order per PR: all checks green -> start review (pending) -> approve ->
# submit the approved review -> merge.
#
# The approval is posted with APPROVE_TOKEN (OWNER_TOKEN, so it comes from the
# repository owner; its body says it is automated); the merge uses GH_TOKEN
# (OWNER_TOKEN) so the push to main triggers the usual workflows.
#
#   GH_TOKEN=... APPROVE_TOKEN=... GITHUB_REPOSITORY=owner/repo [DRY_RUN=true] scripts/approve-ready-prs.sh
set -euo pipefail

: "${GH_TOKEN:?GH_TOKEN is required}" "${APPROVE_TOKEN:?APPROVE_TOKEN is required}" "${GITHUB_REPOSITORY:?}"
repo=$GITHUB_REPOSITORY
dry_run=${DRY_RUN:-false}
# Account that posts the approval: the owner with OWNER_TOKEN; GITHUB_TOKEN
# cannot read /user, so that case is github-actions.
approver=$(GH_TOKEN=$APPROVE_TOKEN gh api user --jq .login 2>/dev/null || echo github-actions)

prs=$(gh pr list --repo "$repo" --state open --limit 200 \
  --json number,author,isDraft \
  --jq '.[] | select(.author.is_bot and (.isDraft | not)) | .number')

for pr in $prs; do
  info=$(gh pr view "$pr" --repo "$repo" \
    --json title,mergeable,files,statusCheckRollup,reviews,additions,deletions)
  title=$(jq -r .title <<<"$info")

  if [[ $(jq -r .mergeable <<<"$info") != MERGEABLE ]]; then
    echo "#$pr: not mergeable yet, skipping"
    continue
  fi
  # Checks: commit statuses report .state, check runs report .status/.conclusion.
  pending=$(jq '[.statusCheckRollup[] | select((.status // "COMPLETED") != "COMPLETED" or (.state // "") == "PENDING")] | length' <<<"$info")
  failed=$(jq '[.statusCheckRollup[] | select((.conclusion // .state) as $c | ["FAILURE","ERROR","CANCELLED","TIMED_OUT","ACTION_REQUIRED","STARTUP_FAILURE"] | index($c))] | length' <<<"$info")
  total=$(jq '.statusCheckRollup | length' <<<"$info")
  if (( total == 0 || pending > 0 || failed > 0 )); then
    echo "#$pr: checks not ready ($total total, $pending pending, $failed failed), skipping"
    continue
  fi

  approved=$(jq --arg me "$approver" '[.reviews[] | select(.author.login == $me and .state == "APPROVED")] | length' <<<"$info")
  if (( approved == 0 )); then
    files=$(jq -r '.files[] | "- `\(.path)` (+\(.additions)/-\(.deletions))"' <<<"$info")
    body=$(printf 'Automated approval: all %s checks passed and the branch has no conflicts.\n\nChanged files:\n%s\n\n_Posted by %s; no person reviewed this change._' \
      "$total" "$files" ".github/workflows/approve-ready-prs.yml")
    echo "#$pr: starting review ($title)"
    if [[ $dry_run != true ]]; then
      # 1. Start a review: a review created without an event stays PENDING.
      #    Reuse a pending review left by an earlier, interrupted run
      #    (GitHub allows one pending review per user per PR).
      review_id=$(GH_TOKEN=$APPROVE_TOKEN gh api "repos/$repo/pulls/$pr/reviews" \
        --jq '[.[] | select(.state == "PENDING")][0].id // empty')
      if [[ -z $review_id ]]; then
        review_id=$(GH_TOKEN=$APPROVE_TOKEN gh api -X POST "repos/$repo/pulls/$pr/reviews" \
          -f body="$body" --jq .id)
      fi
      # 2-3. Approve and submit the pending review.
      echo "#$pr: submitting approved review $review_id"
      GH_TOKEN=$APPROVE_TOKEN gh api -X POST "repos/$repo/pulls/$pr/reviews/$review_id/events" \
        -f event=APPROVE -f body="$body" >/dev/null
    fi
  fi

  echo "#$pr: merging"
  # --auto also works when a merge queue is required: the PR joins the queue.
  [[ $dry_run == true ]] || gh pr merge "$pr" --repo "$repo" --squash --auto --subject "$title (#$pr)" --body "" \
    || echo "#$pr: merge failed, will retry next run"
done
