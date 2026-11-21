#!/usr/bin/env bash
# Finds every Cargo, Go and npm project in the repository and opens one
# auto-merging PR per dependency version that is newer than the locked one:
#   - Cargo: each locked package `cargo update --dry-run` would change
#   - Go:    each direct module with an update (`go get <mod>@<new>`)
#   - npm:   each direct dependency whose locked version is behind the latest release
#
# Each PR targets one exact version (branch deps/<project>/<eco>-<pkg>-<version>).
# When a newer version appears while an older PR for the same dependency is
# still open, the older PR is merged first (or closed as superseded if its
# checks failed) and a new PR is opened for the newer version on top of it.
#
# Usage: BASE=main GH_TOKEN=... scripts/update-deps.sh
set -euo pipefail

BASE=${BASE:-main}
opened=0

sync_base() {
  git fetch --quiet origin "$BASE"
  git switch --quiet --force --detach "origin/$BASE"
  git clean -qfd
}

# semver_label OLD NEW: deps/major, deps/minor or deps/patch. As in Cargo's
# semver rules, a 0.x minor bump (0.4 -> 0.5) counts as breaking (major).
semver_label() {
  local old=${1#v} new=${2#v} oM om nM nm
  IFS=. read -r oM om _ <<<"${old%%[-+]*}"
  IFS=. read -r nM nm _ <<<"${new%%[-+]*}"
  if [[ "$oM" != "$nM" ]] || [[ "$oM" == 0 && "$om" != "$nm" ]]; then echo deps/major
  elif [[ "$om" != "$nm" ]]; then echo deps/minor
  else echo deps/patch
  fi
}

# supersede PREFIX BRANCH: merge (or close) open PRs for older versions of the
# same dependency. Returns 1 if the exact BRANCH already has an open PR.
supersede() {
  local prefix=$1 branch=$2 rows number head failed
  rows=$(gh pr list --state open --label kind/deps --limit 200 \
    --json number,headRefName,statusCheckRollup \
    --jq ".[] | select(.headRefName | startswith(\"$prefix\")) |
      \"\(.number) \(.headRefName) \([.statusCheckRollup[]? | select(.conclusion == \"FAILURE\" or .conclusion == \"TIMED_OUT\")] | length)\"")
  while read -r number head failed; do
    [[ -n "$number" ]] || continue
    [[ "$head" == "$branch" ]] && return 1
    if [[ "$failed" -gt 0 ]]; then
      gh pr close "$number" --delete-branch --comment "Superseded by a newer version (\`$branch\`); this one had failing checks."
      gh pr edit "$number" --add-label status/superseded || true
    elif gh pr view "$number" --json labels --jq '.labels[].name' | grep -qx deps/major; then
      gh pr close "$number" --delete-branch --comment "Superseded by a newer version (\`$branch\`)."
      gh pr edit "$number" --add-label status/superseded || true
    else
      # Merge now so the newer version lands on top; --admin bypasses checks still pending.
      gh pr merge "$number" --rebase --delete-branch --admin || gh pr merge "$number" --rebase --delete-branch \
        || echo "::warning::Could not merge superseded PR #$number"
    fi
  done <<<"$rows"
  sync_base
  return 0
}

# propose DIR ECO PKG OLD NEW LANG UPDATE_CMD...: apply one update and open its PR.
propose() {
  local dir=$1 eco=$2 pkg=$3 old=$4 new=$5 lang=$6
  shift 6
  local name slugpkg prefix branch
  name=$([[ "$dir" == "." ]] && echo root || basename "$dir")
  [[ "$eco" == nix ]] && name=flake
  # Labels use the project names from scripts/labels.mjs: the root workspace is
  # the CLI and go/ is the activity tool.
  local app=$name
  case "$dir" in . | flake) app=cli ;; go) app=activity ;; esac
  slugpkg=${pkg//\//-}
  prefix="deps/$name/$eco-$slugpkg-"
  branch="$prefix${new#v}"

  supersede "$prefix" "$branch" || return 0
  (cd "$dir" && "$@") >/dev/null || return 0
  git diff --quiet && return 0

  local title="chore($name/deps): $pkg $old -> $new" semver
  # Flake inputs are pinned to commits, not semver releases.
  if [[ "$eco" == nix ]]; then semver=deps/minor; else semver=$(semver_label "$old" "$new"); fi
  git add -A
  git commit --quiet -m "$title"
  git push --quiet --force origin "HEAD:refs/heads/$branch"
  gh pr create --base "$BASE" --head "$branch" --title "$title" \
    --body "Automated update of \`$pkg\` ($old -> $new) in \`$dir\`." \
    --label kind/deps --label "lang/$lang" --label "app/$app" --label source/update-deps \
    --label "$semver" && opened=$((opened + 1))
  # Every update, majors included, auto-merges once required checks pass (needs
  # "Allow auto-merge" in the repository settings). This also runs when the PR
  # already existed and its branch was just updated. Squash, like
  # dependabot-auto-merge.yml, so the PR title becomes the commit on main.
  gh pr merge "$branch" --auto --squash --subject "$title" --body "" ||
    echo "::warning::Auto-merge unavailable; $branch left open."
}

projects() { git ls-files -- "*$1" | xargs -r -n1 dirname | sort -u; }

sync_base

for dir in $(projects Cargo.lock); do
  changes=$(cd "$dir" && cargo update --dry-run 2>&1 | sed -nE 's/^ *Updating ([^ ]+) (v[^ ]+) -> (v[^ ]+).*/\1 \2 \3/p')
  while read -r pkg old new; do
    [[ -n "$pkg" ]] || continue
    propose "$dir" cargo "$pkg" "$old" "$new" rust cargo update --quiet -p "$pkg@${old#v}" --precise "${new#v}"
  done <<<"$changes"
done

for dir in $(projects go.mod); do
  changes=$(cd "$dir" && go list -m -u -f '{{if and (not .Main) (not .Indirect) .Update}}{{.Path}} {{.Version}} {{.Update.Version}}{{end}}' all) ||
    { echo "::warning::Skipping $dir: go list failed"; continue; }
  while read -r mod old new; do
    [[ -n "$mod" ]] || continue
    propose "$dir" go "$mod" "$old" "$new" go sh -c "go get '$mod@$new' && go mod tidy"
  done <<<"$changes"
done

for dir in $(projects package-lock.json); do
  # Current versions come from the lockfile (CI has no node_modules, so
  # `npm outdated` would report nothing); latest comes from the registry.
  changes=$(jq -r '.packages[""] | ((.dependencies // {}) + (.devDependencies // {})) | keys[]' "$dir/package-lock.json" |
    while read -r pkg; do
      old=$(jq -r --arg p "node_modules/$pkg" '.packages[$p].version // empty' "$dir/package-lock.json")
      new=$(npm view "$pkg" version 2>/dev/null) || continue
      # `if` rather than `[[ ]] && echo`: a false test as the loop's last command
      # would make the pipeline fail and stop the script under `set -e`.
      if [[ -n "$old" && -n "$new" && "$old" != "$new" ]]; then echo "$pkg $old $new"; fi
    done)
  while read -r pkg old new; do
    [[ -n "$pkg" ]] || continue
    dev=$(jq -r --arg p "$pkg" 'if .devDependencies[$p] then "--save-dev" else "--save" end' "$dir/package.json")
    # `npm ls` fails on unmet peer dependencies (e.g. TypeScript 7 under an
    # Angular that requires 6.0), so such updates are skipped, not proposed.
    propose "$dir" npm "$pkg" "v$old" "v$new" js sh -c       "npm install $dev --no-audit --no-fund --ignore-scripts '$pkg@$new' && npm ls --all >/dev/null 2>&1 || { echo '::warning::Skipping $pkg@$new in $dir: unmet peer dependencies'; exit 1; }"
  done <<<"$changes"
done

# Nix flake inputs: one PR per input whose locked commit moved, e.g.
# "chore(flake/deps): nixpkgs a1b2c3d -> e4f5a6b". The first run adds flake.lock.
if [[ -f flake.nix ]] && command -v nix >/dev/null; then
  if [[ ! -f flake.lock ]]; then
    propose . nix flake.lock none initial nix nix flake lock
  else
    for input in $(jq -r '.nodes.root.inputs | keys[]' flake.lock); do
      rev() { jq -r --arg i "$input" '.nodes[.nodes.root.inputs[$i]].locked.rev // empty' flake.lock | cut -c1-7; }
      old=$(rev)
      sync_base
      nix flake update "$input" >/dev/null 2>&1 || continue
      new=$(rev)
      git checkout -- flake.lock
      [[ -n "$old" && -n "$new" && "$old" != "$new" ]] || continue
      propose . nix "$input" "$old" "$new" nix nix flake update "$input"
    done
  fi
fi

# Nix packages in pkgs/ that build a pinned upstream release (a `version` plus
# a source hash in default.nix): nix-update bumps version and hashes, one PR each.
if command -v nix >/dev/null; then
  for file in $(git ls-files -- 'pkgs/*/default.nix'); do
    grep -qE '^\s*version = "' "$file" || continue
    name=$(basename "$(dirname "$file")")
    old=$(sed -nE 's/^\s*version = "([^"]+)";/\1/p' "$file" | head -1)
    sync_base
    nix run nixpkgs#nix-update -- -f nix/update.nix "$name" >/dev/null 2>&1 || continue
    new=$(sed -nE 's/^\s*version = "([^"]+)";/\1/p' "$file" | head -1)
    git checkout -- "pkgs/$name"
    [[ -n "$new" && "$new" != "$old" ]] || continue
    propose "pkgs/$name" nix-update "$name" "v$old" "v$new" nix sh -c "cd ../.. && nix run nixpkgs#nix-update -- -f nix/update.nix $name --version $new"
  done
fi

# Backstage apps (Yarn workspaces) upgrade as a set: bump to the latest release.
for dir in $(projects backstage.json); do
  old=$(jq -r .version "$dir/backstage.json")
  latest=$(curl -fsSL https://versions.backstage.io/v1/tags/main/manifest.json | jq -r .releaseVersion) || continue
  [[ -n "$latest" && "$latest" != "$old" ]] || continue
  propose "$dir" backstage backstage "v$old" "v$latest" js sh -c "corepack enable && yarn backstage-cli versions:bump --release $latest"
done

sync_base
echo "Opened $opened pull request(s)."
