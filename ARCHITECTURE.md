# Architecture

`commit` is a self-hosted **engineering-activity platform**: it collects what happens in
your repositories (pushes, pull requests, CI runs, releases), stores it, analyzes it, and
shows it back to people through dashboards, badges, apps and a weekly report. Every
directory in this monorepo is one part of that pipeline.

```text
 GitHub ──webhooks──▶ webhooks ──▶ events-api ◀── github history: gitstat, crawl
                                       │             offline buffer: event-store
                                       ▼
               analyze: sql · summarize · search · forecast · plugin-host
                                       │
        ┌──────────────────────────────┼──────────────────────────────┐
        ▼                              ▼                              ▼
  show (web/apps)               show (embeds)                  report (weekly)
  dashboard · admin             badges · widget                pipelines → reports-pdf
  portal · website · docs       edge-cache · status-page       → archive → reports
  mobile · desktop · inspector  activity-rpc                   weekly-page · changelog
  runner-monitor · visualizer   shortlinks                     alerts-ui
                                       │
                 operate: helm-render (deploy/helm) · runner-fleet · services
                          tracing · archive-stats · repo-registry · admin-api
```

## Projects

| Stage | Project | What it does | Stack |
|---|---|---|---|
| Collect | `webhooks` | Verifies GitHub webhook signatures, forwards events to `events-api`, Prometheus metrics | Go, gin |
| Collect | `gitstat` | Commits per author straight from a clone | Rust, gitoxide |
| Collect | `crawl` | Release-note/changelog links for dependency PRs | Rust, reqwest, scraper |
| Store | `events-api` | Activity event API over SQLite | Rust, axum, sqlx |
| Store | `event-store` | Offline buffer for events, replayed later | Go, GORM |
| Store | `repo-registry` | Which repositories are tracked | Rust, SeaORM |
| Analyze | `sql` | Ad-hoc SQL over exports | Rust, DataFusion |
| Analyze | `summarize` | Group-by summaries of exports | Rust, polars |
| Analyze | `search` | Full-text search over docs and notes | Rust, tantivy |
| Analyze | `forecast` | Commit-volume trend and projection | Rust, candle |
| Analyze | `plugin-host` | Sandboxed custom metrics as WebAssembly plugins | Rust, wasmtime |
| Serve | `activity-rpc` | Internal "how active is repo X" gRPC service | Rust, tonic |
| Serve | `badges` | README badges (SVG) | Rust, actix-web |
| Serve | `shortlinks` | Short links for shared reports | Rust, Rocket |
| Serve | `edge-cache` | Edge KV cache for badges and status | TypeScript, Hono, Workers |
| Show | `dashboard` | Main web UI (with Storybook) | React, Vite, TanStack Query |
| Show | `admin` + `admin-api` | Admin console and its team/repo ownership API | Angular; NestJS |
| Show | `portal` | Developer portal cataloguing every service here | Backstage |
| Show | `widget` | Embeddable activity widget (minified by `minify`) | Vue |
| Show | `mobile` | CI status on phones | Expo / React Native |
| Show | `desktop` | CI status desktop app | Tauri |
| Show | `inspector` | Native viewer for author exports | Rust, iced |
| Show | `runner-monitor` | Terminal monitor for CI runner hosts | Rust, ratatui |
| Show | `visualizer` | "Office TV" view of author activity | Rust, bevy |
| Show | `alerts-ui` | Alert-rule editor with live preview | Rust, Dioxus |
| Report | `pipelines` | Weekly report workflow | Go, Temporal |
| Report | `reports-pdf` | Renders the report to PDF | Rust, Typst |
| Report | `archive` / `archive-stats` | Upload reports to S3 / report bucket usage | Rust, AWS SDK; Go, AWS SDK v2 |
| Report | `reports` | Browse archived reports | Nuxt |
| Report | `weekly-page` | Server-rendered "this week" page | Rust, Leptos |
| Publish | `website` · `docs` · `changelog` · `status-page` | Landing page, documentation, changelog, public status | Next.js, Docusaurus, SvelteKit, Astro |
| Operate | `helm-render` | Renders `deploy/helm/commit` for review | Go, Helm |
| Operate | `runner-fleet` | CI runner pods on Kubernetes | Go, client-go |
| Operate | `services` | Running platform containers on a Docker host | Go, moby client |
| Operate | `tracing` | Reference OpenTelemetry setup for Go services | Go, OpenTelemetry |
| Build | `minify` | Minifies the widget bundle | Rust, SWC |
| Operate | `vuln-scan` | Scans every lockfile for known vulnerabilities | Nix, OSV-Scanner (pinned) |
| Operate | `sbom` | CycloneDX SBOM per project | Nix, Syft (pinned) |
| Operate | `actions-lint` | Lints workflows for correctness and security | Nix, zizmor (pinned), actionlint |
| Operate | `helm-check` | Lints and schema-validates the deploy chart | Nix, kubeconform (pinned), Helm |
| Operate | `repo-tools` | Repository scripts with pinned dependencies | Nix |
| Operate | `devcontainer` | Codespaces image with every toolchain | Nix, dockerTools |
| Operate | `ci-runner-image` | Ephemeral self-hosted Actions runner image | Nix, dockerTools, github-runner |
| Operate | `nixos-module` | Runs `events-api` and `badges` as systemd services | NixOS module |
| Operate | `integration-test` | Boots the services in a NixOS VM and tests them over HTTP | NixOS test |

The root holds the repository's own tooling: `crates/` (the `commit` validation CLI),
`go/` (commit activity summaries used by the Activity Report workflow),
`web/status-worker` (CI status for the dashboard and apps) and `scripts/`.

## Conventions

- **One lockfile per project.** Each project builds and tests on its own; its CI workflow
  (`.github/workflows/pkg-<name>.yml`) is generated by `scripts/gen-pkg-workflows.mjs`.
- **Labels** follow `<group>/<name>` (`kind/`, `area/`, `app/`, `lang/`, `deps/`,
  `priority/`, `status/`, `effort/`, `source/`), defined in `scripts/labels.mjs`.
  Pull requests get `app/<project>` and `lang/<lang>` automatically.
- **Dependencies** update one version per pull request (`scripts/update-deps.sh`).
  A newer release merges the pending older one first; pull requests waiting more than
  three hours merge in the next `deps-merge.yml` run (every 4 hours). Majors wait for review.
- The portal catalog (`pkgs/portal/examples/entities.yaml`) is generated from this
  directory by `scripts/gen-catalog.mjs`.
