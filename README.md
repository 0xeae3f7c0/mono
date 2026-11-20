# commit

A self-maintaining GitHub repository automation toolkit written in Rust. It maintains **its own repository** by running checks, reviewing proposed changes, and opening maintenance pull requests when real tracked-file changes are available.

It does not fabricate contribution history, create empty pull requests, impersonate human reviewers, or approve its own changes.

## Features

- Rust command-line application.
- Repository structure and whitespace validation.
- CI checks for formatting, Clippy and tests.
- Automated review summaries on pull requests.
- Scheduled maintenance PRs for formatting and Cargo lockfile updates.
- Dependabot (daily) for Cargo dependencies and GitHub Actions, with auto-merge for patch and minor updates.
- Manual maintenance runs through the Actions tab.

## Layout

Monorepo with three toolchains:

```text
crates/
  cli/      `commit` binary (clap)
  core/     repository validation rules
  git/      git helpers
  report/   report type, text and JSON output
go/
  cmd/activity/        commit-per-author summary
  internal/activity/   summary logic
web/
  status-worker/       Cloudflare Worker serving /health and /status
pkgs/                  standalone projects, each with its own lockfile and CI workflow
scripts/               dependency updates, labels, generators
.github/workflows/     CI, automation and janitor workflows
```

## Projects (`pkgs/`)

`commit` is a self-hosted engineering-activity platform; each project in `pkgs/` is one
part of it (collect, store, analyze, show, report, operate). See
[ARCHITECTURE.md](ARCHITECTURE.md) for the map and what each project does.

Each project is self-contained (own `Cargo.lock`, `go.sum`, `package-lock.json` or
`yarn.lock`) and has a generated workflow `.github/workflows/pkg-<name>.yml`. After adding
or removing a project run `node scripts/gen-pkg-workflows.mjs` and
`node scripts/gen-catalog.mjs`; CI fails if either is stale.

`node scripts/count-deps.mjs` prints unique locked dependencies per ecosystem.

## Run locally

Install stable Rust from https://rustup.rs.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -- check

(cd go && go test ./...)
(cd web/status-worker && npm test)
```

Review changed files against a fetched base branch:

```bash
git fetch origin main
cargo run -- review origin/main
```

## GitHub Actions

- **CI:** formatting, Clippy, tests and repository validation on pushes and PRs.
- **Automated Code Review:** runs the same deterministic checks and posts a summary comment. It reports findings; it does not approve PRs or replace human review.
- **Repository Maintenance:** every 4 hours, runs `cargo update -p <package>` for each locked Cargo package, `go get -u` for each direct Go module, and bumps each npm devDependency in `web/status-worker`, and opens one auto-merging PR per package with a newer release (`chore(deps/<package>): vOLD -> vNEW`). Keeps one tracking issue open while runs fail.
- **Auto Format:** runs `cargo fmt` after every push to `main` and commits only when formatting changes files.
- **Stale Run Sweeper:** every 4 hours, cancels queued runs older than 6 hours whose commit is no longer the branch head. Manual runs default to dry run.

Set an `OWNER_TOKEN` secret (owner PAT with `repo` scope) so automated PRs and pushes trigger CI and are attributed to the owner. Without it, workflows fall back to `GITHUB_TOKEN`. For auto-merge, enable **Settings → General → Allow auto-merge** and require the `Rust checks` status check on `main`.
- **Go CI / Web CI:** path-filtered checks for `go/` and `web/` (Web CI runs ESLint, Prettier, tests and a `wrangler` dry-run build).
- **CodeQL:** Rust, Go, JavaScript and Actions analysis on push, PR and weekly.
- **Nightly:** full release-mode tests on Linux, macOS and Windows, plus docs, Go and web.
- **Release:** pushing a `v*` tag builds the CLI for three platforms and publishes a GitHub release.
- **Workflow Lint:** `actionlint` on workflow changes.
- **Labeler:** labels PRs by area (`rust`, `go`, `web`, `ci`, `scripts`, `docs`).
- **Labels:** syncs `<group>/<name>` labels from `scripts/labels.mjs`.
- **Merge Waiting Dependency PRs:** every 4 hours, merges dependency PRs older than 3 hours with no failing checks (majors excluded).
- **pkg/<name>:** one generated CI workflow per project.
- **Stale / Branch Cleanup:** closes inactive issues and PRs; deletes merged automation branches.
- **Activity Report:** every 4 hours, commits-per-author summary from the Go tool.
- **Dependabot:** checks GitHub Actions daily; Cargo, Go and npm updates come from Repository Maintenance.

A maintenance run with no changes creates no pull request.

## Required repository settings

In **Settings → Actions → General → Workflow permissions**:

1. Select **Read and write permissions**.
2. Enable **Allow GitHub Actions to create and approve pull requests** if you want the maintenance workflow to create PRs. This project creates PRs but does not approve them.
3. Save.

For `main`, consider a branch ruleset requiring pull requests, the `Rust checks` status check and at least one human approval. Do not allow the PR author to satisfy their own approval requirement.

## Maintenance behaviour

The scheduled workflow does not invent code changes. It can produce a PR when formatting or dependency resolution changes tracked files. Dependabot handles version update proposals. Add future maintenance tasks as explicit, testable operations.

## License

MIT. See [LICENSE](LICENSE).
