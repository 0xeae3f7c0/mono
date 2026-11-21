#!/usr/bin/env bash
# Regenerate per-project workflows, the labeler config and the portal catalog.
# Shortcuts: g, ge, gen. Use `./gen --check` to only verify they are current.
set -euo pipefail
cd "$(dirname "$0")"
node scripts/gen-pkg-workflows.mjs "$@"
node scripts/gen-catalog.mjs "$@"
