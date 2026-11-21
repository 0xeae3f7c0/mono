#!/usr/bin/env node
// Generates one CI workflow per project under pkgs/ (.github/workflows/pkg-<name>.yml),
// picking the toolchain from the project's manifest, plus .github/labeler.yml so
// pull requests get `app/<project>`, `lang/<lang>` and `area/<area>` labels.
// Re-run after adding or removing a project; `--check` fails if any generated
// file is out of date.
//
//   node scripts/gen-pkg-workflows.mjs [--check]

import { existsSync, readdirSync, readFileSync, writeFileSync, unlinkSync } from "node:fs";
import { join } from "node:path";

const check = process.argv.includes("--check");
const pkgs = readdirSync("pkgs", { withFileTypes: true }).filter((d) => d.isDirectory()).map((d) => d.name).sort();

// Extra apt packages some Rust projects need on ubuntu-latest.
const apt = {
  visualizer: "pkg-config libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev libx11-dev",
  desktop: "libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev",
};

const hasTf = (dir) => readdirSync(dir).some((f) => f.endsWith(".tf"));

function steps(name, dir) {
  if (existsSync(join(dir, "Cargo.toml"))) {
    return [
      ...(apt[name] ? [`      - run: sudo apt-get update && sudo apt-get install -y ${apt[name]}`] : []),
      "      - uses: dtolnay/rust-toolchain@stable",
      "        with:",
      "          components: clippy",
      "      - uses: Swatinem/rust-cache@v2",
      "        with:",
      `          workspaces: ${dir}`,
      "      - run: cargo clippy --locked --all-targets -- -D warnings",
      "      - run: cargo test --locked",
    ];
  }
  if (existsSync(join(dir, "go.mod"))) {
    return [
      "      - uses: actions/setup-go@v5",
      "        with:",
      `          go-version-file: ${dir}/go.mod`,
      `          cache-dependency-path: ${dir}/go.sum`,
      '      - run: test -z "$(gofmt -l .)"',
      "      - run: go vet ./...",
      "      - run: go test ./...",
    ];
  }
  if (existsSync(join(dir, "yarn.lock"))) {
    return [
      "      - uses: actions/setup-node@v4",
      "        with:",
      "          node-version: 24",
      "      - run: corepack enable",
      "      - run: yarn install --immutable",
      "      - run: yarn ci",
    ];
  }
  if (existsSync(join(dir, "package.json"))) {
    const pkg = JSON.parse(readFileSync(join(dir, "package.json"), "utf8"));
    const deps = Object.keys({ ...pkg.dependencies, ...pkg.devDependencies });
    // Browser tests (e.g. Storybook via @vitest/browser-playwright) need the
    // Playwright browser that matches the installed version, plus its system libraries.
    const playwright = deps.some((d) => d === "playwright" || d === "@playwright/test" || d === "@vitest/browser-playwright");
    return [
      "      - uses: actions/setup-node@v4",
      "        with:",
      "          node-version: 24",
      "          cache: npm",
      `          cache-dependency-path: ${dir}/package-lock.json`,
      "      - run: npm ci",
      ...(playwright ? ["      - run: npx playwright install --with-deps chromium"] : []),
      "      - run: npm run ci",
    ];
  }
  if (existsSync(join(dir, "pyproject.toml"))) {
    return [
      "      - uses: astral-sh/setup-uv@v7",
      "        with:",
      "          enable-cache: true",
      `          cache-dependency-glob: ${dir}/uv.lock`,
      "      - run: uv sync --locked",
      "      - run: uv run ruff check",
      "      - run: uv run ruff format --check",
      "      - run: uv run pytest",
    ];
  }
  // OpenTofu / Terraform configurations: formatted and validated, never applied.
  if (hasTf(dir)) {
    return [
      "      - uses: opentofu/setup-opentofu@v2",
      "      - run: tofu fmt -check -recursive",
      "      - run: tofu init -backend=false -input=false",
      "      - run: tofu validate",
    ];
  }
  // Nix projects: a package (default.nix) is built from the flake; the NixOS
  // module (module.nix) is checked by evaluating it.
  if (existsSync(join(dir, "default.nix")) || existsSync(join(dir, "module.nix"))) {
    const check = existsSync(join(dir, "default.nix"))
      ? `nix build "../..#${name}" --print-build-logs`
      : `nix eval "../..#nixosModules.default" --apply builtins.isFunction`;
    // NixOS VM tests need /dev/kvm, which hosted runners have but lock down.
    const vmTest =
      existsSync(join(dir, "default.nix")) && /runNixOSTest/.test(readFileSync(join(dir, "default.nix"), "utf8"));
    const kvm = [
      "      - name: Enable KVM",
      "        run: |",
      `          echo 'KERNEL=="kvm", GROUP="kvm", MODE="0666", OPTIONS+="static_node=kvm"' | sudo tee /etc/udev/rules.d/99-kvm4all.rules`,
      "          sudo udevadm control --reload-rules && sudo udevadm trigger --name-match=kvm",
    ];
    return [...(vmTest ? kvm : []), "      - uses: DeterminateSystems/nix-installer-action@v20", `      - run: ${check}`];
  }
  return null;
}

// Paths outside the project that should also trigger its workflow.
const extraPaths = { "helm-render": ["deploy/helm/**"] };

function workflow(name) {
  const dir = `pkgs/${name}`;
  const body = steps(name, dir);
  if (!body) return null;
  // Nix projects also rebuild when the flake or the shared Nix files change.
  const nixPaths = lang(dir) === "nix" ? ["flake.nix", "flake.lock", "nix/**"] : [];
  const paths = `[${[`${dir}/**`, ...(extraPaths[name] ?? []), ...nixPaths, `.github/workflows/pkg-${name}.yml`].map((p) => `"${p}"`).join(", ")}]`;
  return [
    "# Generated by scripts/gen-pkg-workflows.mjs. Do not edit by hand.",
    `name: pkg/${name}`,
    "",
    "on:",
    "  push:",
    "    branches: [main]",
    `    paths: ${paths}`,
    "  pull_request:",
    `    paths: ${paths}`,
    "  workflow_dispatch:",
    "",
    "permissions:",
    "  contents: read",
    "",
    "concurrency:",
    `  group: pkg-${name}-\${{ github.ref }}`,
    "  cancel-in-progress: ${{ github.event_name == 'pull_request' }}",
    "",
    "jobs:",
    "  test:",
    `    name: ${name}`,
    "    runs-on: ubuntu-latest",
    "    defaults:",
    "      run:",
    `        working-directory: ${dir}`,
    "    steps:",
    "      - uses: actions/checkout@v4",
    ...body,
    "",
  ].join("\n");
}

function lang(dir) {
  if (existsSync(join(dir, "Cargo.toml"))) return "rust";
  if (existsSync(join(dir, "go.mod"))) return "go";
  if (existsSync(join(dir, "package.json"))) return "js";
  if (existsSync(join(dir, "pyproject.toml"))) return "python";
  if (hasTf(dir)) return "terraform";
  if (existsSync(join(dir, "default.nix")) || existsSync(join(dir, "module.nix"))) return "nix";
  return null;
}

function labeler() {
  const rule = (globs) =>
    `  - changed-files:\n      - any-glob-to-any-file: [${globs.map((g) => `"${g}"`).join(", ")}]`;
  const entries = [
    ["area/ci", [".github/**"]],
    ["area/scripts", ["scripts/**"]],
    ["area/docs", ["**/*.md", "pkgs/docs/**"]],
    ["area/cli", ["crates/**", "Cargo.toml", "Cargo.lock"]],
    ["app/cli", ["crates/**"]],
    ["app/activity", ["go/**"]],
    ["app/status-worker", ["web/status-worker/**"]],
  ];
  const byLang = { rust: ["crates/**"], go: ["go/**"], js: ["web/**"], python: [], terraform: [], nix: ["flake.nix", "nix/**"] };
  for (const name of pkgs) {
    const l = lang(`pkgs/${name}`);
    if (!l) continue;
    entries.push([`app/${name}`, [`pkgs/${name}/**`]]);
    byLang[l].push(`pkgs/${name}/**`);
  }
  for (const [l, globs] of Object.entries(byLang)) entries.push([`lang/${l}`, globs]);
  entries.sort((a, b) => a[0].localeCompare(b[0]));
  return [
    "# Generated by scripts/gen-pkg-workflows.mjs from the projects in pkgs/. Do not edit by hand.",
    ...entries.map(([n, g]) => `${n}:\n${rule(g)}`),
    "",
  ].join("\n");
}

// Keep action versions in step with what Dependabot has already bumped: use
// the highest major found for each action across the existing workflows.
const pinned = {};
for (const f of readdirSync(".github/workflows")) {
  for (const [, action, major] of readFileSync(`.github/workflows/${f}`, "utf8").matchAll(/uses: ([\w./-]+)@v(\d+)/g)) {
    pinned[action] = Math.max(pinned[action] ?? 0, Number(major));
  }
}
const pin = (text) => text.replace(/uses: ([\w./-]+)@v(\d+)/g, (m, a, v) => `uses: ${a}@v${Math.max(Number(v), pinned[a] ?? 0)}`);

let stale = 0;
const wanted = new Set();
{
  const file = ".github/labeler.yml";
  const text = labeler();
  if ((existsSync(file) ? readFileSync(file, "utf8") : null) !== text) {
    stale++;
    if (check) console.error(`out of date: ${file}`);
    else writeFileSync(file, text);
  }
}
for (const name of pkgs) {
  const raw = workflow(name);
  if (!raw) continue;
  const text = pin(raw);
  const file = `.github/workflows/pkg-${name}.yml`;
  wanted.add(file);
  const current = existsSync(file) ? readFileSync(file, "utf8") : null;
  if (current === text) continue;
  stale++;
  if (check) console.error(`out of date: ${file}`);
  else writeFileSync(file, text);
}
for (const file of readdirSync(".github/workflows").map((f) => `.github/workflows/${f}`)) {
  if (/\/pkg-[^/]+\.yml$/.test(file) && !wanted.has(file)) {
    stale++;
    if (check) console.error(`orphaned: ${file}`);
    else unlinkSync(file);
  }
}
if (check && stale) process.exit(1);
console.log(`${wanted.size} project workflows${check ? " checked" : ` written (${stale} changed)`}.`);
