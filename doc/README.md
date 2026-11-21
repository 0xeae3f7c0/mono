# Documentation

Everything that used to live in the top-level README, in one place.

- [Layout](#layout)
- [Projects](#projects)
- [Run locally](#run-locally) (including the root shortcut scripts)
- [Automation](#automation)
- [Repository settings](#repository-settings)

Related: [`ARCHITECTURE.md`](../ARCHITECTURE.md) (what each project does and how they
fit together), [`CONTRIBUTING.md`](../CONTRIBUTING.md) (how a change lands),
[`SECURITY.md`](../SECURITY.md) (reporting vulnerabilities).

## Layout

```text
crates/               the `commit` validation CLI (Rust workspace: cli, core, git, report)
go/                   commit-per-author summary used by the Activity Report workflow
web/status-worker/    Cloudflare Worker serving /health and /status
pkgs/                 the platform's projects, each with its own lockfile and CI workflow
deploy/helm/commit/   Helm chart for the platform services
scripts/              dependency updates, labels and generators
.github/workflows/    CI, automation and housekeeping workflows
```

## Projects

`commit` is a self-hosted engineering-activity platform; each project in `pkgs/` is one
part of it (collect, store, analyze, show, report, operate). See
[`ARCHITECTURE.md`](../ARCHITECTURE.md) for the map.

Each project is self-contained (its own `Cargo.lock`, `go.sum`, `package-lock.json` or
`yarn.lock`) and has a generated workflow, `.github/workflows/pkg-<name>.yml`. After
adding or removing a project, run:

```bash
node scripts/gen-pkg-workflows.mjs   # per-project workflows and .github/labeler.yml
node scripts/gen-catalog.mjs         # Backstage catalog in pkgs/portal
```

CI fails if either output is stale. `node scripts/count-deps.mjs` prints unique locked
dependencies per ecosystem.

**Nix projects** are folders with a `default.nix` (a package) or `module.nix` (the
NixOS module). The flake finds them automatically (`nix/pkgs.nix`), so
`nix build .#<name>` works for each. Packages that build a pinned upstream release
(`vuln-scan`, `sbom`, `actions-lint`, `helm-check`) set `version` and source hashes in
`default.nix`; Repository Maintenance runs `nix-update` (through `nix/update.nix`) and
opens one pull request per new release. Container images (`devcontainer`,
`ci-runner-image` and the service images) are pushed to GitHub Container Registry by
`publish-images.yml`.

## Run locally

Install stable Rust (https://rustup.rs), Go and Node.js 22+.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -- check

(cd go && go test ./...)
(cd web/status-worker && npm test)
```

Each project in `pkgs/` runs on its own: `cargo test` (Rust), `go test ./...` (Go),
`npm ci && npm run ci` (npm) or `yarn install && yarn ci` (`pkgs/portal`).

Shortcuts at the root do the same; any prefix of the name works. They are symlinks
to `check.sh`, `gen.sh` and `deps.sh` (on Windows, enable `core.symlinks` or use
the `.sh` names):

```bash
./check            # or ./c, ./ch, ./che, ./chec: root checks
./check webhooks   # one project's checks
./gen --check      # or ./g, ./ge: generated files are current
./deps             # or ./d, ./de, ./dep: unique dependency counts
```

Review changed files against a fetched base branch:

```bash
git fetch origin main
cargo run -- review origin/main
```

## Automation

Times are UTC. Workflows with a schedule can also be started from the Actions tab.

### On every change

| Workflow | What it does |
|---|---|
| CI | Root CLI: format, Clippy, tests, repository validation; checks generated files are current |
| pkg/&lt;name&gt; | One per project, only when that project changes: Clippy and tests, `go vet`/`go test`, or `npm run ci` |
| Go CI / Web CI | `go/` and `web/status-worker` checks |
| Workflow Lint | `actionlint` on workflow changes |
| Automated Code Review | Rust checks on pull requests, then a summary comment (not an approval) |
| Auto Format | `cargo fmt` after pushes to `main`; commits only if something changed |
| Labeler | `app/<project>`, `lang/<lang>` and `area/<area>` labels on pull requests |
| CodeQL | Security scan of Rust, Go, JS/TS and Actions on pushes to `main` and weekly |

### Dependencies

| Workflow | When | What it does |
|---|---|---|
| Repository Maintenance | every 4 hours | One pull request per dependency version across every Cargo, Go, npm and Backstage project, and per Nix flake input. Patch and minor updates auto-merge; npm updates that break peer dependencies are skipped. |
| Merge Waiting Dependency PRs | every 4 hours | Merges dependency pull requests older than 3 hours with no failing checks. Majors wait for review. |
| Dependabot | daily | GitHub Actions versions; patch and minor updates squash-merge automatically. |
| Dependabot Alerts to Issues | every 4 hours | One issue per open security alert (`kind/security`, `app/<project>`, priority by severity); closes when the alert is fixed or dismissed. |
| Code Scanning Alerts to Issues | every 4 hours | One issue per open code scanning alert (CodeQL, Scorecard; `kind/security`, `source/code-scanning`, `app/<project>`, priority by severity); closes when the alert is fixed or dismissed. |

### Housekeeping

| Workflow | When | What it does |
|---|---|---|
| Nightly | daily 01:07 | Release-mode tests on Linux, macOS and Windows, docs, Go and web |
| Activity Report | every 4 hours | Commits per author for the period, in the run summary |
| Stale | daily 03:41 | Marks and later closes inactive issues and pull requests |
| Stale Run Sweeper | every 4 hours | Cancels queued runs for outdated commits |
| Branch Cleanup | Sundays 04:11 | Deletes branches of closed automation pull requests |
| Labels | on change, Mondays | Syncs `<group>/<name>` labels from `scripts/labels.mjs` |
| Release | on `v*` tags | Builds the CLI for Linux, macOS and Windows and publishes a release |

### Pull requests and releases

| Workflow or app | What it does |
|---|---|
| Release Please | Keeps a `chore(release): vX.Y.Z` PR with the changelog; merging it tags the release, which `release.yml` builds |
| PR Title | Fails pull requests whose title is not a Conventional Commit |
| PR Size | Labels pull requests `size/xs` … `size/xl` (lockfiles ignored) |
| Coverage | Rust (`crates/`) and Go coverage, uploaded to Codecov |
| Scorecard | Weekly OpenSSF security-practice score, in Security → Code scanning |
| Merge queue | Ruleset setting; `ci.yml` runs on `merge_group` |

GitHub Apps (install once; their settings are in the repository root):

| App | Config | Install |
|---|---|---|
| CodeRabbit (AI review) | `.coderabbit.yaml` | https://github.com/apps/coderabbitai |
| Socket (supply-chain alerts) | `socket.yml` | https://github.com/apps/socket-security |
| Codecov (coverage comments) | `codecov.yml` | https://github.com/apps/codecov |
| ImgBot (image compression) | `.imgbotconfig` | https://github.com/apps/imgbot |

### External trigger (cron-job.org)

GitHub drops scheduled runs when it is busy. Schedule Watchdog
(`schedule-watchdog.yml`) re-dispatches any 4-hourly workflow that is overdue, and
an external cron at https://cron-job.org starts the watchdog itself, so it runs even
when GitHub skips its own schedule.

1. Create a fine-grained personal access token: repository access **only
   `0xeae3f7c0/mono`**, permission **Actions: Read and write**, nothing else. Set an
   expiry and renew it.
2. In cron-job.org, create a job:

   | Field | Value |
   |---|---|
   | URL | `https://api.github.com/repos/0xeae3f7c0/mono/actions/workflows/schedule-watchdog.yml/dispatches` |
   | Schedule | every hour, minute 35 (between the watchdog's own :05 runs) |
   | Request method | `POST` |
   | Headers | `Authorization: Bearer <token>`, `Accept: application/vnd.github+json`, `X-GitHub-Api-Version: 2026-03-10` |
   | Body | `{"ref":"main"}` |

3. Use **Test run**: a `204` response means the watchdog was dispatched.

The watchdog only dispatches workflows whose last run is 4 or more hours old, so
extra triggers never cause duplicate maintenance runs.

## Repository settings

1. Add an `OWNER_TOKEN` secret (personal access token with `repo` scope) so automated
   pull requests and pushes trigger CI.
2. **Settings → General:** enable **Allow auto-merge**; under pull requests, untick
   **Allow merge commits** and set the squash default message to **Pull request title**.
3. **Settings → Actions → General → Workflow permissions:** read and write, and allow
   GitHub Actions to create pull requests.
4. **Settings → Rules:** a ruleset on `main` requiring the `Rust checks` and
   `Generated files` status checks, so auto-merge waits for them.
5. **Merge queue:** in the same ruleset, turn on **Require merge queue** (merge method
   squash, "Only merge non-failing pull requests"). Auto-merging pull requests then
   join the queue and land one at a time, each tested against the latest `main`, so
   dependency updates to the same lockfile no longer conflict. `ci.yml` runs on the
   `merge_group` event for this.
