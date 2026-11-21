#!/usr/bin/env bash
# Format, lint and test the root tools, then validate the repository.
# Shortcuts: c, ch, che, chec, check. Pass a project name to check only it: ./check webhooks
set -euo pipefail
cd "$(dirname "$0")"

if [[ $# -gt 0 ]]; then
  dir="pkgs/$1"
  [[ -d "$dir" ]] || { echo "no such project: $1" >&2; exit 2; }
  cd "$dir"
  if [[ -f Cargo.toml ]]; then cargo clippy --all-targets -- -D warnings && cargo test
  elif [[ -f go.mod ]]; then go vet ./... && go test ./...
  elif [[ -f yarn.lock ]]; then yarn ci
  else npm run ci
  fi
  exit
fi

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -q -- check
(cd go && go vet ./... && go test ./...)
(cd web/status-worker && npm test)
