#!/usr/bin/env bash
# Print unique locked dependencies per ecosystem (Cargo, Go, npm, Yarn).
# Shortcuts: d, de, dep, deps. Add --json for machine-readable output.
set -euo pipefail
cd "$(dirname "$0")"
node scripts/count-deps.mjs . "$@"
