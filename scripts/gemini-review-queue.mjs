#!/usr/bin/env node
// Asks Gemini Code Assist to review open pull requests one at a time: it posts
// `/gemini review` on the oldest pull request whose current head has no Gemini
// review yet, then waits until that review arrives before moving on.
//
// A request counts as in progress until Gemini submits a review after it, or
// for TIMEOUT_MINUTES; a request that times out is not repeated for the same
// head commit, so one stuck pull request never blocks the queue. Pull requests
// that only change files matching ignore_patterns in .gemini/config.yaml
// (lockfiles and the like) are skipped without a comment.
//
//   GH_TOKEN=... GITHUB_REPOSITORY=owner/repo [DRY_RUN=true] node scripts/gemini-review-queue.mjs

import { readFileSync } from "node:fs";
import { nextLink } from "./alerts-to-issues.mjs";

export const GEMINI = "gemini-code-assist[bot]";
export const COMMAND = "/gemini review";
const TIMEOUT_MINUTES = 30;

/** ignore_patterns from .gemini/config.yaml (a flat list of quoted globs). */
export function ignorePatterns(yaml) {
  const block = yaml.replaceAll("\r", "").match(/^ignore_patterns:\n((?:[ \t]*(?:-.*|#.*)?\n)*)/m)?.[1] ?? "";
  return [...block.matchAll(/^\s*-\s*["']?([^"'#\n]+?)["']?\s*$/gm)].map((m) => m[1]);
}

// Glob to RegExp: a leading "**" plus slash matches any directories (or none);
// "*" matches within one path segment.
export function globToRegExp(glob) {
  const source = glob
    .split(/(\*\*\/|\*)/)
    .map((part) => (part === "**/" ? "(?:.*/)?" : part === "*" ? "[^/]*" : part.replace(/[.+?^${}()|[\]\\]/g, "\\$&")))
    .join("");
  return new RegExp(`^${source}$`);
}

/** True when every changed file matches an ignore pattern. */
export function onlyIgnored(files, patterns) {
  const regexps = patterns.map(globToRegExp);
  return files.length > 0 && files.every((f) => regexps.some((r) => r.test(f)));
}

/**
 * Decides what to do with one pull request.
 * Returns "pending" (a request is still in progress), "review" (ask now) or "skip".
 */
export function stateOf({ headSha, headDate, reviews, requests, now = Date.now() }) {
  const geminiReviews = reviews.filter((r) => r.user?.login === GEMINI && r.submitted_at);
  const lastReview = Math.max(0, ...geminiReviews.map((r) => Date.parse(r.submitted_at)));
  const lastRequest = Math.max(0, ...requests.map((c) => Date.parse(c.created_at)));
  if (lastRequest > lastReview && now - lastRequest < TIMEOUT_MINUTES * 60_000) return "pending";
  if (geminiReviews.some((r) => r.commit_id === headSha)) return "skip";
  // Already asked about this head and it timed out: do not ask again.
  if (lastRequest > Date.parse(headDate)) return "skip";
  return "review";
}

async function main() {
  const { GH_TOKEN, GITHUB_REPOSITORY } = process.env;
  const dryRun = process.env.DRY_RUN === "true";
  if (!GH_TOKEN || !GITHUB_REPOSITORY) throw new Error("GH_TOKEN and GITHUB_REPOSITORY are required");
  const base = `https://api.github.com/repos/${GITHUB_REPOSITORY}`;
  const request = async (url, init = {}) => {
    const res = await fetch(url, {
      ...init,
      headers: {
        authorization: `Bearer ${GH_TOKEN}`,
        accept: "application/vnd.github+json",
        "user-agent": "gemini-review-queue",
      },
    });
    if (!res.ok) throw new Error(`${init.method ?? "GET"} ${url}: ${res.status} ${await res.text()}`);
    return res;
  };
  const api = async (path, init = {}) => (await request(`${base}${path}`, init)).json();
  const all = async (path) => {
    const out = [];
    let url = `${base}${path}${path.includes("?") ? "&" : "?"}per_page=100`;
    while (url) {
      const res = await request(url);
      out.push(...(await res.json()));
      url = nextLink(res.headers.get("link"));
    }
    return out;
  };

  const patterns = ignorePatterns(readFileSync(".gemini/config.yaml", "utf8"));
  const pulls = (await all("/pulls?state=open&sort=created&direction=asc")).filter((p) => !p.draft);
  let next = null;
  for (const pr of pulls) {
    const files = (await all(`/pulls/${pr.number}/files`)).map((f) => f.filename);
    if (onlyIgnored(files, patterns)) continue;
    const [reviews, comments, head] = await Promise.all([
      all(`/pulls/${pr.number}/reviews`),
      all(`/issues/${pr.number}/comments`),
      api(`/commits/${pr.head.sha}`),
    ]);
    const state = stateOf({
      headSha: pr.head.sha,
      headDate: head.commit.committer.date,
      reviews,
      requests: comments.filter((c) => c.body.trim() === COMMAND),
    });
    if (state === "pending") {
      console.log(`#${pr.number}: Gemini review in progress; waiting.`);
      return;
    }
    if (state === "review" && !next) next = pr;
  }
  if (!next) {
    console.log(`${pulls.length} open pull requests; none need a Gemini review.`);
    return;
  }
  console.log(`#${next.number}: requesting a Gemini review${dryRun ? " (dry run)" : ""}.`);
  if (!dryRun) await api(`/issues/${next.number}/comments`, { method: "POST", body: JSON.stringify({ body: COMMAND }) });
}

if (import.meta.url === `file://${process.argv[1].replaceAll("\\", "/").replace(/^(?=[A-Za-z]:)/, "/")}`) {
  await main();
}
