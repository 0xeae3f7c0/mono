# Common tasks (https://just.systems). Run `just` to list them.

default:
    @just --list

# Format, lint and test the root tools, then validate the repository
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo run -q -- check
    cd go && go vet ./... && go test ./...
    cd web/status-worker && npm test

# Regenerate per-project workflows, labeler config and the portal catalog
gen:
    node scripts/gen-pkg-workflows.mjs
    node scripts/gen-catalog.mjs

# Print unique locked dependencies per ecosystem
deps:
    node scripts/count-deps.mjs .

# Run one project's checks, e.g. `just pkg webhooks`
pkg name:
    cd pkgs/{{name}} && if [ -f Cargo.toml ]; then cargo clippy --all-targets -- -D warnings && cargo test; elif [ -f go.mod ]; then go vet ./... && go test ./...; elif [ -f yarn.lock ]; then yarn ci; else npm run ci; fi
