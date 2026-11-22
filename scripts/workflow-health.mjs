#!/usr/bin/env node
// Opens one issue per workflow problem and closes it once the problem is gone.
// Two sources:
//   - actionlint findings in .github/workflows/ (pass its JSON with --lint FILE).
//     One issue per finding, keyed by file, rule and message (not line, so
//     edits above a finding do not reopen it as a new issue).
//   - Failing workflows: the latest finished run on main of each active
//     workflow. One issue per workflow; it closes when a later run succeeds.
//   - Failing pull requests: the latest finished run of each workflow on the
//     head commit of each open pull request. One issue per pull request,
//     listing its failing workflows; it closes when the current head has no
//     failing runs, or when the pull request is merged or closed.
// Each issue carries a hidden marker (<!-- workflow-lint:KEY -->,
// <!-- workflow-failure:ID --> or <!-- workflow-pr-failure:N -->) so reruns
// update, never duplicate.
//
//   actionlint -format '{{json .}}' > lint.json
//   GH_TOKEN=... GITHUB_REPOSITORY=owner/repo [DRY_RUN=true] node scripts/workflow-health.mjs --lint lint.json

import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { nextLink } from "./alerts-to-issues.mjs";

const LINT_MARKER = /<!-- workflow-lint:([0-9a-f]+) -->/;
const FAILURE_MARKER = /<!-- workflow-failure:(\d+) -->/;
const PR_MARKER = /<!-- workflow-pr-failure:(\d+) -->/;
const LABELS = ["kind/ci", "area/ci", "source/workflow-health", "status/needs-triage"];
const FAILED = new Set(["failure", "timed_out", "startup_failure"]);
// Runs that say nothing about whether the workflow works.
const IGNORED = new Set(["cancelled", "skipped", "neutral", "stale", "action_required"]);

/** Stable key for one actionlint finding. */
export function lintKey(finding) {
  return createHash("sha256").update(`${finding.filepath}\n${finding.kind}\n${finding.message}`).digest("hex").slice(0, 16);
}

/** Issue title, labels and body for one actionlint finding. */
export function lintIssueFor(finding, repo, sha) {
  const where = `${finding.filepath}:${finding.line}`;
  const body = [
    `<!-- workflow-lint:${lintKey(finding)} -->`,
    `**actionlint \`${finding.kind}\`** in \`${where}\``,
    "",
    "```",
    finding.message,
    "```",
    "",
    finding.snippet ? ["```yaml", finding.snippet, "```", ""].join("\n") : "",
    `File: https://github.com/${repo}/blob/${sha}/${finding.filepath}#L${finding.line}`,
    "",
    "_This issue closes automatically when actionlint no longer reports this finding._",
  ].join("\n");
  return { title: `fix(ci): ${finding.kind} in ${finding.filepath}`, body, labels: [...LABELS, "priority/normal"] };
}

/** Issue title, labels and body for a workflow whose latest run on main failed. */
export function failureIssueFor(workflow, run) {
  const body = [
    `<!-- workflow-failure:${workflow.id} -->`,
    `**${workflow.name}** (\`${workflow.path}\`) failed on main.`,
    "",
    "| | |",
    "|---|---|",
    `| Run | [#${run.run_number}](${run.html_url}) |`,
    `| Conclusion | ${run.conclusion} |`,
    `| Event | ${run.event} |`,
    `| Commit | \`${run.head_sha.slice(0, 7)}\` |`,
    `| Finished | ${run.updated_at} |`,
    "",
    "_This issue closes automatically when a later run of this workflow on main succeeds._",
  ].join("\n");
  return { title: `fix(ci): ${workflow.name} workflow failing on main`, body, labels: [...LABELS, "priority/high"] };
}

/** Latest finished run of each workflow, ignoring runs that say nothing (cancelled, skipped, ...). */
export function latestRuns(runs) {
  const latest = new Map();
  for (const run of runs) {
    if (run.status !== "completed" || IGNORED.has(run.conclusion)) continue;
    const seen = latest.get(run.workflow_id);
    if (!seen || Date.parse(run.updated_at) > Date.parse(seen.updated_at)) latest.set(run.workflow_id, run);
  }
  return [...latest.values()];
}

/** Issue title, labels and body for a pull request whose head has failing runs. */
export function prFailureIssueFor(pr, failed) {
  const body = [
    `<!-- workflow-pr-failure:${pr.number} -->`,
    `Pull request #${pr.number} (**${pr.title}**) has failing workflows on its head commit \`${pr.head.sha.slice(0, 7)}\`.`,
    "",
    "| Workflow | Run | Conclusion | Event |",
    "|---|---|---|---|",
    ...failed
      .sort((a, b) => a.name.localeCompare(b.name))
      .map((run) => `| ${run.name} | [#${run.run_number}](${run.html_url}) | ${run.conclusion} | ${run.event} |`),
    "",
    "_This issue closes automatically when the pull request's head commit has no failing runs, or when it is merged or closed._",
  ].join("\n");
  return { title: `fix(ci): workflows failing on #${pr.number}`, body, labels: [...LABELS, "priority/normal"] };
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function main() {
  const { GH_TOKEN, GITHUB_REPOSITORY, GITHUB_SHA = "main" } = process.env;
  const dryRun = process.env.DRY_RUN === "true";
  const lintArg = process.argv.indexOf("--lint");
  if (lintArg === -1) throw new Error("--lint FILE (actionlint JSON output) is required");
  if (!GH_TOKEN || !GITHUB_REPOSITORY) throw new Error("GH_TOKEN and GITHUB_REPOSITORY are required");
  const findings = JSON.parse(readFileSync(process.argv[lintArg + 1], "utf8") || "[]");

  const base = `https://api.github.com/repos/${GITHUB_REPOSITORY}`;
  // Writes are spaced a second apart to stay under the secondary rate limit.
  const request = async (url, init = {}) => {
    if ((init.method ?? "GET") !== "GET") await sleep(1000);
    const res = await fetch(url, {
      ...init,
      headers: {
        authorization: `Bearer ${GH_TOKEN}`,
        accept: "application/vnd.github+json",
        "user-agent": "workflow-health",
      },
    });
    if (!res.ok) throw new Error(`${init.method ?? "GET"} ${url}: ${res.status} ${await res.text()}`);
    return res;
  };
  const api = async (path, init = {}) => {
    const res = await request(`${base}${path}`, init);
    return res.status === 204 ? null : res.json();
  };
  // `key` names the array in endpoints that wrap it in an object, such as
  // { total_count, workflows: [...] } from /actions/workflows.
  const all = async (path, key) => {
    const out = [];
    let url = `${base}${path}${path.includes("?") ? "&" : "?"}per_page=100`;
    while (url) {
      const res = await request(url);
      const page = await res.json();
      out.push(...(key ? page[key] : page));
      url = nextLink(res.headers.get("link"));
    }
    return out;
  };

  const issues = (await all("/issues?state=all&labels=source/workflow-health")).filter((i) => !i.pull_request);
  const byLint = new Map();
  const byFailure = new Map();
  const byPull = new Map();
  for (const issue of issues) {
    const pull = issue.body?.match(PR_MARKER);
    if (pull) byPull.set(Number(pull[1]), issue);
    const lint = issue.body?.match(LINT_MARKER);
    const failure = issue.body?.match(FAILURE_MARKER);
    if (lint) byLint.set(lint[1], issue);
    if (failure) byFailure.set(Number(failure[1]), issue);
  }

  const counts = { created: 0, reopened: 0, updated: 0, closed: 0 };
  // Opens, reopens or refreshes the issue for a current problem.
  const report = async (existing, issue) => {
    if (!existing) {
      console.log(`create: ${issue.title}`);
      if (!dryRun) await api("/issues", { method: "POST", body: JSON.stringify(issue) });
      counts.created++;
    } else if (existing.state === "closed") {
      console.log(`reopen #${existing.number}: ${issue.title}`);
      if (!dryRun) await api(`/issues/${existing.number}`, { method: "PATCH", body: JSON.stringify({ state: "open", body: issue.body }) });
      counts.reopened++;
    } else if (existing.body !== issue.body) {
      if (!dryRun) await api(`/issues/${existing.number}`, { method: "PATCH", body: JSON.stringify({ body: issue.body }) });
      counts.updated++;
    }
  };
  const resolve = async (issue, comment) => {
    console.log(`close #${issue.number}: ${comment}`);
    if (!dryRun) {
      await api(`/issues/${issue.number}/comments`, { method: "POST", body: JSON.stringify({ body: comment }) });
      await api(`/issues/${issue.number}`, { method: "PATCH", body: JSON.stringify({ state: "closed", state_reason: "completed" }) });
    }
    counts.closed++;
  };

  // actionlint findings.
  const current = new Set();
  for (const finding of findings) {
    const key = lintKey(finding);
    if (current.has(key)) continue;
    current.add(key);
    await report(byLint.get(key), lintIssueFor(finding, GITHUB_REPOSITORY, GITHUB_SHA));
  }
  for (const [key, issue] of byLint) {
    if (!current.has(key) && issue.state === "open") {
      await resolve(issue, `actionlint no longer reports this finding at \`${GITHUB_SHA.slice(0, 7)}\`; closing.`);
    }
  }

  // Latest finished run on main of each active workflow.
  // More than 100 workflows exist, so every page is read.
  const workflows = await all("/actions/workflows", "workflows");
  let failing = 0;
  for (const workflow of workflows.filter((w) => w.state === "active")) {
    const { workflow_runs: runs } = await api(`/actions/workflows/${workflow.id}/runs?branch=main&status=completed&per_page=20`);
    const run = runs.find((r) => !IGNORED.has(r.conclusion));
    if (!run) continue;
    const existing = byFailure.get(workflow.id);
    if (FAILED.has(run.conclusion)) {
      failing++;
      await report(existing, failureIssueFor(workflow, run));
    } else if (existing?.state === "open") {
      await resolve(existing, `${workflow.name} passed on main in [run #${run.run_number}](${run.html_url}); closing.`);
    }
  }

  // Latest finished run of each workflow on the head commit of each open pull request.
  const pulls = await all("/pulls?state=open");
  const open = new Set(pulls.map((pr) => pr.number));
  let failingPulls = 0;
  for (const pr of pulls) {
    const runs = await all(`/actions/runs?head_sha=${pr.head.sha}`, "workflow_runs");
    const failed = latestRuns(runs).filter((run) => FAILED.has(run.conclusion));
    const existing = byPull.get(pr.number);
    if (failed.length) {
      failingPulls++;
      await report(existing, prFailureIssueFor(pr, failed));
    } else if (existing?.state === "open" && runs.length) {
      await resolve(existing, `No failing runs on #${pr.number} at \`${pr.head.sha.slice(0, 7)}\`; closing.`);
    }
  }
  for (const [number, issue] of byPull) {
    if (!open.has(number) && issue.state === "open") {
      await resolve(issue, `#${number} is merged or closed; closing.`);
    }
  }

  console.log(
    `${current.size} lint findings, ${failing} failing workflows, ${failingPulls} failing pull requests: ${counts.created} issues created, ` +
      `${counts.reopened} reopened, ${counts.updated} updated, ${counts.closed} closed${dryRun ? " (dry run)" : ""}.`,
  );
}

if (import.meta.url === `file://${process.argv[1].replaceAll("\\", "/").replace(/^(?=[A-Za-z]:)/, "/")}`) {
  await main();
}
