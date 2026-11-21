#!/usr/bin/env node
// repository tooling (scripts/)
// Single source of truth for repository labels, using `<group>/<name>` names.
// `app/<project>` labels are derived from the projects in pkgs/ plus the root parts.
//
//   node scripts/labels.mjs            print the label set as JSON
//   node scripts/labels.mjs --sync     create/update labels on GitHub
//                                      (needs GH_TOKEN and GITHUB_REPOSITORY)

import { readdirSync, existsSync } from "node:fs";
import { join } from "node:path";

const groups = {
  kind: ["0e8a16", {
    bug: "Something is broken",
    feature: "New capability",
    enhancement: "Improvement to existing behaviour",
    chore: "Maintenance with no behaviour change",
    deps: "Dependency update",
    ci: "Workflows and automation",
    docs: "Documentation",
    refactor: "Restructuring without behaviour change",
    performance: "Speed or resource use",
    security: "Security fix or hardening",
  }],
  area: ["1d76db", {
    ci: ".github/ and workflows",
    docs: "Markdown and the docs site",
    scripts: "scripts/",
    cli: "crates/ (the commit CLI)",
    infra: "Deployment, containers, Kubernetes",
    web: "Browser and edge front ends",
    data: "Storage, analytics and search",
  }],
  lang: ["5319e7", { rust: "Rust code", go: "Go code", js: "JavaScript / TypeScript code", nix: "Nix expressions" }],
  deps: ["c5def5", { patch: "Patch version bump", minor: "Minor version bump", major: "Major version bump (manual review)" }],
  priority: ["d93f0b", { critical: "Drop everything", high: "Next up", normal: "Default", low: "When time allows" }],
  status: ["fbca04", {
    "needs-triage": "Not yet reviewed",
    "in-progress": "Being worked on",
    blocked: "Waiting on something else",
    "needs-review": "Ready for review",
    "awaiting-ci": "Waiting for checks",
    superseded: "Replaced by a newer pull request",
  }],
  effort: ["bfdadc", { xs: "Minutes", s: "An hour or two", m: "About a day", l: "Several days" }],
  // Set by .github/workflows/pr-size.yml from changed lines (lockfiles ignored).
  size: ["c2e0c6", { xs: "Up to 10 lines", s: "Up to 100 lines", m: "Up to 500 lines", l: "Up to 1000 lines", xl: "Over 1000 lines" }],
  source: ["ededed", {
    human: "Opened by a person",
    "update-deps": "Opened by scripts/update-deps.sh",
    dependabot: "Opened by Dependabot",
    maintenance: "Opened by a maintenance workflow",
  }],
};

/** Projects that get an `app/<name>` label. */
export function apps() {
  const pkgs = existsSync("pkgs")
    ? readdirSync("pkgs", { withFileTypes: true }).filter((d) => d.isDirectory()).map((d) => d.name)
    : [];
  return ["cli", "activity", "status-worker", ...pkgs].sort();
}

export function labels() {
  const out = [];
  for (const [group, [color, names]] of Object.entries(groups)) {
    for (const [name, description] of Object.entries(names)) out.push({ name: `${group}/${name}`, color, description });
  }
  for (const app of apps()) out.push({ name: `app/${app}`, color: "006b75", description: `Project: ${app}` });
  return out;
}

async function sync() {
  const { GH_TOKEN, GITHUB_REPOSITORY } = process.env;
  if (!GH_TOKEN || !GITHUB_REPOSITORY) throw new Error("GH_TOKEN and GITHUB_REPOSITORY are required");
  const api = (path, init = {}) =>
    fetch(`https://api.github.com/repos/${GITHUB_REPOSITORY}${path}`, {
      ...init,
      headers: { authorization: `Bearer ${GH_TOKEN}`, accept: "application/vnd.github+json", "user-agent": "labels-sync" },
    });

  const existing = new Map();
  for (let page = 1; ; page++) {
    const batch = await (await api(`/labels?per_page=100&page=${page}`)).json();
    for (const l of batch) existing.set(l.name, l);
    if (batch.length < 100) break;
  }
  for (const label of labels()) {
    const current = existing.get(label.name);
    if (current && current.color === label.color && current.description === label.description) continue;
    const res = current
      ? await api(`/labels/${encodeURIComponent(label.name)}`, { method: "PATCH", body: JSON.stringify(label) })
      : await api("/labels", { method: "POST", body: JSON.stringify(label) });
    if (!res.ok) throw new Error(`${label.name}: ${res.status} ${await res.text()}`);
    console.log(`${current ? "updated" : "created"} ${label.name}`);
  }
}

if (import.meta.url === `file://${process.argv[1].replaceAll("\\", "/").replace(/^(?=[A-Za-z]:)/, "/")}`) {
  if (process.argv.includes("--sync")) await sync();
  else console.log(JSON.stringify(labels(), null, 2));
}
