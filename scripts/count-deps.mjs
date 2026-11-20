#!/usr/bin/env node
// Counts unique locked dependencies (name@version) per ecosystem across every
// lockfile under a directory. Skips node_modules, target and .git.
//
//   node scripts/count-deps.mjs [dir] [--json]

import { readdirSync, readFileSync } from "node:fs";
import { join, basename, relative } from "node:path";

const root = process.argv.slice(2).find((a) => !a.startsWith("--")) ?? ".";
const asJson = process.argv.includes("--json");
const skip = new Set(["node_modules", "target", ".git", "dist", ".svelte-kit", ".angular", ".docusaurus"]);

const parsers = {
  "Cargo.lock": [
    "cargo",
    (t) => [...t.matchAll(/^name = "([^"]+)"\r?\nversion = "([^"]+)"/gm)].map((m) => `${m[1]}@${m[2]}`),
  ],
  "go.sum": [
    "go",
    (t) =>
      t
        .split("\n")
        .filter(Boolean)
        .map((l) => l.split(" "))
        .map(([mod, ver]) => `${mod}@${ver.replace("/go.mod", "")}`),
  ],
  "package-lock.json": [
    "npm",
    (t) =>
      Object.entries(JSON.parse(t).packages ?? {})
        .filter(([k, v]) => k && !v.link && v.version)
        .map(([k, v]) => `${k.split("node_modules/").pop()}@${v.version}`),
  ],
  "bun.lock": ["npm", (t) => [...t.matchAll(/^ {4}"[^"]+": \["([^"]+)"/gm)].map((m) => m[1])],
  // Yarn Berry: `resolution: "name@npm:1.2.3"` (workspace and patch entries skipped).
  "yarn.lock": [
    "npm",
    (t) => [...t.matchAll(/^ {2}resolution: "(.+)@npm:([^"]+)"/gm)].map((m) => `${m[1]}@${m[2]}`),
  ],
  "flake.lock": ["nix", (t) => Object.keys(JSON.parse(t).nodes).filter((n) => n !== "root")],
};

function* lockfiles(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.isDirectory() && !skip.has(entry.name)) yield* lockfiles(join(dir, entry.name));
    else if (entry.isFile() && parsers[entry.name]) yield join(dir, entry.name);
  }
}

const stats = {};
for (const file of lockfiles(root)) {
  const [kind, parse] = parsers[basename(file)];
  const names = parse(readFileSync(file, "utf8"));
  const s = (stats[kind] ??= { files: [], unique: new Set() });
  s.files.push(relative(root, file).replaceAll("\\", "/"));
  // Nix inputs are only meaningful per flake, so key them by file.
  for (const n of names) s.unique.add(kind === "nix" ? `${file}:${n}` : n);
}

const rows = Object.entries(stats).map(([kind, s]) => ({ kind, lockfiles: s.files.length, unique: s.unique.size }));
const total = rows.reduce((n, r) => n + r.unique, 0);
if (asJson) {
  console.log(JSON.stringify({ total, ecosystems: rows }, null, 2));
} else {
  for (const r of rows) console.log(`${r.kind.padEnd(6)} ${String(r.lockfiles).padStart(3)} lockfiles ${String(r.unique).padStart(6)} unique`);
  console.log(`total  ${String(total).padStart(20)} unique`);
}
