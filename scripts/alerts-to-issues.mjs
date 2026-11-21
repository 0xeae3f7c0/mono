#!/usr/bin/env node
// Mirrors open security alerts into issues and closes them once the alerts are
// fixed or dismissed. Two sources:
//   - Dependabot alerts (the default): one issue per alert.
//   - Code scanning alerts (CodeQL, Scorecard, ...): one issue per tool and rule,
//     listing every open alert for it, because one rule can raise dozens.
// Each issue carries a hidden marker (<!-- dependabot-alert:N --> or
// <!-- code-scanning-rule:Tool/Rule -->) so reruns update, never duplicate.
//
//   GH_TOKEN=... GITHUB_REPOSITORY=owner/repo [DRY_RUN=true] node scripts/alerts-to-issues.mjs [--source code-scanning]

const MARKER = /<!-- dependabot-alert:(\d+) -->/;
const SEVERITY_PRIORITY = {
  critical: "priority/critical",
  high: "priority/high",
  medium: "priority/normal",
  low: "priority/low",
  // Code scanning rules without a security severity use these levels.
  error: "priority/high",
  warning: "priority/normal",
  note: "priority/low",
};

/** `app/<project>` label for the manifest an alert was raised against. */
export function appLabel(manifest) {
  const [top, name] = manifest.split("/");
  if (top === "pkgs" && name) return `app/${name}`;
  if (top === "web") return "app/status-worker";
  if (top === "go") return "app/activity";
  return "app/cli";
}

/** URL of the `rel="next"` page in a GitHub `Link` header, or null. */
export function nextLink(header) {
  const match = header?.split(",").find((part) => /rel="next"/.test(part))?.match(/<([^>]+)>/);
  return match ? match[1] : null;
}

/** Issue title, labels and body for one alert. */
export function issueFor(alert) {
  const pkg = alert.dependency.package;
  const manifest = alert.dependency.manifest_path;
  const advisory = alert.security_advisory;
  const vuln = alert.security_vulnerability;
  const severity = advisory.severity;
  const patched = vuln.first_patched_version?.identifier;
  const title = `fix(security): ${pkg.name} ${severity} vulnerability (${advisory.ghsa_id})`;
  const body = [
    `<!-- dependabot-alert:${alert.number} -->`,
    `**${advisory.summary}**`,
    "",
    "| | |",
    "|---|---|",
    `| Package | \`${pkg.name}\` (${pkg.ecosystem}) |`,
    `| Manifest | \`${manifest}\` |`,
    `| Severity | ${severity} |`,
    `| Vulnerable | \`${vuln.vulnerable_version_range}\` |`,
    `| Fixed in | ${patched ? `\`${patched}\`` : "no patched version yet"} |`,
    `| Advisory | [${advisory.ghsa_id}](https://github.com/advisories/${advisory.ghsa_id}) |`,
    "",
    `Dependabot alert: ${alert.html_url}`,
    "",
    patched
      ? `Fix: update \`${pkg.name}\` to \`${patched}\` or later. Repository Maintenance opens this update automatically when it is compatible.`
      : "No fixed release exists yet; consider a workaround or removing the dependency.",
    "",
    "_This issue closes automatically when the alert is fixed or dismissed._",
  ].join("\n");
  const labels = [
    "kind/security",
    "source/dependabot",
    "status/needs-triage",
    appLabel(manifest),
    SEVERITY_PRIORITY[severity] ?? "priority/normal",
  ];
  return { title, body, labels };
}

/** Grouping key for a code scanning alert: one issue per tool and rule. */
export const ruleKey = (alert) => `${alert.tool.name}/${alert.rule.id}`;

const SEVERITY_ORDER = ["critical", "high", "error", "medium", "warning", "low", "note"];
const MAX_LOCATIONS = 100;

/**
 * Issue title, labels and body for one code scanning rule, listing every open
 * alert raised by it. Scorecard alone raises dozens of alerts per rule (one per
 * unpinned action), so one issue per alert would flood the tracker.
 */
export function codeScanningIssueFor(alerts) {
  const [first] = alerts;
  const rule = first.rule;
  const severities = alerts.map((a) => a.rule.security_severity_level ?? a.rule.severity ?? "warning");
  const severity = SEVERITY_ORDER.find((s) => severities.includes(s)) ?? "warning";
  const rows = alerts
    .map((a) => {
      const location = a.most_recent_instance?.location ?? {};
      const where = location.path ? `${location.path}${location.start_line ? `:${location.start_line}` : ""}` : "unknown";
      return { path: location.path ?? "", line: `- [ ] [\`${where}\`](${a.html_url})` };
    })
    .sort((x, y) => x.line.localeCompare(y.line));
  const apps = [...new Set(rows.map((r) => appLabel(r.path)))].sort();
  const title = `fix(security): ${rule.description ?? rule.id} (${first.tool.name} ${rule.id})`;
  const body = [
    `<!-- code-scanning-rule:${ruleKey(first)} -->`,
    `**${rule.description ?? rule.id}**`,
    "",
    "| | |",
    "|---|---|",
    `| Tool | ${first.tool.name} |`,
    `| Rule | \`${rule.id}\` |`,
    `| Severity | ${severity} |`,
    `| Open alerts | ${alerts.length} |`,
    "",
    ...rows.slice(0, MAX_LOCATIONS).map((r) => r.line),
    ...(rows.length > MAX_LOCATIONS ? [`- … and ${rows.length - MAX_LOCATIONS} more in Security → Code scanning`] : []),
    "",
    "_This issue is updated as alerts change and closes when none are left open._",
  ].join("\n");
  const labels = [
    "kind/security",
    "source/code-scanning",
    "status/needs-triage",
    ...apps,
    SEVERITY_PRIORITY[severity] ?? "priority/normal",
  ];
  return { title, body, labels };
}

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function main() {
  const { GH_TOKEN, GITHUB_REPOSITORY } = process.env;
  const dryRun = process.env.DRY_RUN === "true";
  const sourceArg = process.argv.indexOf("--source");
  const sourceName = sourceArg === -1 ? "dependabot" : process.argv[sourceArg + 1];
  if (!["dependabot", "code-scanning"].includes(sourceName)) {
    throw new Error(`unknown --source ${sourceName}; expected dependabot or code-scanning`);
  }
  if (!GH_TOKEN || !GITHUB_REPOSITORY) throw new Error("GH_TOKEN and GITHUB_REPOSITORY are required");

  const base = `https://api.github.com/repos/${GITHUB_REPOSITORY}`;
  // GitHub's secondary rate limit blocks bursts of content creation, so writes
  // are spaced a second apart and a rate-limited request waits and retries.
  const request = async (url, init = {}) => {
    const write = (init.method ?? "GET") !== "GET";
    for (let attempt = 1; ; attempt++) {
      if (write) await sleep(1000);
      const res = await fetch(url, {
        ...init,
        headers: {
          authorization: `Bearer ${GH_TOKEN}`,
          accept: "application/vnd.github+json",
          "user-agent": "alerts-to-issues",
        },
      });
      if (res.ok) return res;
      const text = await res.text();
      const limited = res.status === 429 || (res.status === 403 && /rate limit/i.test(text));
      if (limited && attempt < 4) {
        const wait = Number(res.headers.get("retry-after")) || 60 * attempt;
        console.log(`rate limited; retrying in ${wait}s`);
        await sleep(wait * 1000);
        continue;
      }
      throw new Error(`${init.method ?? "GET"} ${url}: ${res.status} ${text}`);
    }
  };
  const api = async (path, init = {}) => {
    const res = await request(`${base}${path}`, init);
    return res.status === 204 ? null : res.json();
  };
  // Follow `Link: rel="next"`; the Dependabot alerts API pages by cursor and
  // rejects `page=` numbers, while issues and code scanning accept either.
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

  if (sourceName === "code-scanning") return syncCodeScanning({ api, all, dryRun });

  const alerts = await all("/dependabot/alerts?state=open");
  const issues = (await all("/issues?state=all&labels=source/dependabot,kind/security")).filter((i) => !i.pull_request);
  const byAlert = new Map();
  for (const issue of issues) {
    const m = issue.body?.match(MARKER);
    if (m) byAlert.set(Number(m[1]), issue);
  }

  let created = 0;
  let reopened = 0;
  let closed = 0;
  const open = new Set(alerts.map((a) => a.number));
  for (const alert of alerts) {
    const existing = byAlert.get(alert.number);
    const issue = issueFor(alert);
    if (!existing) {
      console.log(`create: ${issue.title}`);
      if (!dryRun) await api("/issues", { method: "POST", body: JSON.stringify(issue) });
      created++;
    } else if (existing.state === "closed") {
      console.log(`reopen #${existing.number}: alert ${alert.number} is open again`);
      if (!dryRun) await api(`/issues/${existing.number}`, { method: "PATCH", body: JSON.stringify({ state: "open" }) });
      reopened++;
    }
  }
  for (const [number, issue] of byAlert) {
    if (open.has(number) || issue.state === "closed") continue;
    const alert = await api(`/dependabot/alerts/${number}`);
    const reason = alert.state === "fixed" ? "completed" : "not_planned";
    console.log(`close #${issue.number}: alert ${number} is ${alert.state}`);
    if (!dryRun) {
      await api(`/issues/${issue.number}/comments`, {
        method: "POST",
        body: JSON.stringify({ body: `Dependabot alert ${number} is now **${alert.state}**; closing.` }),
      });
      await api(`/issues/${issue.number}`, { method: "PATCH", body: JSON.stringify({ state: "closed", state_reason: reason }) });
    }
    closed++;
  }
  console.log(
    `${alerts.length} open alerts: ${created} issues created, ${reopened} reopened, ${closed} closed${dryRun ? " (dry run)" : ""}.`,
  );
}

const RULE_MARKER = /<!-- code-scanning-rule:(\S+) -->/;
// Issues from the first version of this script, one per alert. They are closed
// in favour of the per-rule issue; nothing is deleted.
const ALERT_MARKER = /<!-- code-scanning-alert:(\d+) -->/;

/** One issue per code scanning rule with open alerts; closes rules with none left. */
async function syncCodeScanning({ api, all, dryRun }) {
  const alerts = await all("/code-scanning/alerts?state=open");
  const issues = (await all("/issues?state=all&labels=source/code-scanning,kind/security")).filter((i) => !i.pull_request);
  const byRule = new Map();
  const legacy = [];
  for (const issue of issues) {
    const m = issue.body?.match(RULE_MARKER);
    if (m) byRule.set(m[1], issue);
    else if (ALERT_MARKER.test(issue.body ?? "") && issue.state === "open") legacy.push(issue);
  }

  const groups = new Map();
  for (const alert of alerts) {
    const key = ruleKey(alert);
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(alert);
  }

  let created = 0;
  let updated = 0;
  let closed = 0;
  const numbers = new Map();
  const titles = new Map();
  for (const [key, group] of groups) {
    const issue = codeScanningIssueFor(group);
    titles.set(issue.title, key);
    const existing = byRule.get(key);
    if (!existing) {
      console.log(`create: ${issue.title} (${group.length} alerts)`);
      if (!dryRun) numbers.set(key, (await api("/issues", { method: "POST", body: JSON.stringify(issue) })).number);
      created++;
    } else if (existing.state === "closed" || existing.body !== issue.body) {
      console.log(`update #${existing.number}: ${key} has ${group.length} open alerts`);
      const patch = { body: issue.body, ...(existing.state === "closed" ? { state: "open" } : {}) };
      if (!dryRun) await api(`/issues/${existing.number}`, { method: "PATCH", body: JSON.stringify(patch) });
      numbers.set(key, existing.number);
      updated++;
    } else {
      numbers.set(key, existing.number);
    }
  }
  for (const [key, issue] of byRule) {
    if (groups.has(key) || issue.state === "closed") continue;
    console.log(`close #${issue.number}: no open ${key} alerts`);
    if (!dryRun) {
      await api(`/issues/${issue.number}/comments`, {
        method: "POST",
        body: JSON.stringify({ body: `No open code scanning alerts remain for \`${key}\`; closing.` }),
      });
      await api(`/issues/${issue.number}`, { method: "PATCH", body: JSON.stringify({ state: "closed", state_reason: "completed" }) });
    }
    closed++;
  }
  for (const issue of legacy) {
    // Old and new issues share a title format, so the title finds the rule.
    const target = numbers.get(titles.get(issue.title));
    console.log(`close #${issue.number}: replaced by the per-rule issue${target ? ` #${target}` : ""}`);
    if (!dryRun) {
      await api(`/issues/${issue.number}/comments`, {
        method: "POST",
        body: JSON.stringify({ body: `Code scanning alerts are now tracked one issue per rule${target ? `: see #${target}` : ""}. Closing.` }),
      });
      await api(`/issues/${issue.number}`, { method: "PATCH", body: JSON.stringify({ state: "closed", state_reason: "not_planned" }) });
    }
    closed++;
  }
  console.log(
    `${alerts.length} open alerts in ${groups.size} rules: ${created} issues created, ${updated} updated, ${closed} closed${dryRun ? " (dry run)" : ""}.`,
  );
}

if (import.meta.url === `file://${process.argv[1].replaceAll("\\", "/").replace(/^(?=[A-Za-z]:)/, "/")}`) {
  await main();
}
