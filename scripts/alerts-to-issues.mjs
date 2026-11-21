#!/usr/bin/env node
// Mirrors open security alerts into issues, one issue per alert, and closes the
// issue once the alert is fixed or dismissed. Two sources: Dependabot alerts
// (the default) and code scanning alerts (CodeQL, Scorecard, ...). Each issue
// carries a hidden marker (<!-- dependabot-alert:N --> or <!-- code-scanning-alert:N -->)
// so reruns update, never duplicate.
//
//   GH_TOKEN=... GITHUB_REPOSITORY=owner/repo [DRY_RUN=true] node scripts/alerts-to-issues.mjs [--source code-scanning]

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

/** Issue title, labels and body for one code scanning alert. */
export function codeScanningIssueFor(alert) {
  const rule = alert.rule;
  const severity = rule.security_severity_level ?? rule.severity ?? "warning";
  const location = alert.most_recent_instance?.location ?? {};
  const path = location.path ?? "";
  const where = path ? `\`${path}${location.start_line ? `:${location.start_line}` : ""}\`` : "unknown";
  const message = alert.most_recent_instance?.message?.text;
  const title = `fix(security): ${rule.description ?? rule.id} (${alert.tool.name} ${rule.id})`;
  const body = [
    `<!-- code-scanning-alert:${alert.number} -->`,
    `**${rule.description ?? rule.id}**`,
    "",
    "| | |",
    "|---|---|",
    `| Tool | ${alert.tool.name} |`,
    `| Rule | \`${rule.id}\` |`,
    `| Severity | ${severity} |`,
    `| Location | ${where} |`,
    "",
    ...(message ? [message, ""] : []),
    `Code scanning alert: ${alert.html_url}`,
    "",
    "_This issue closes automatically when the alert is fixed or dismissed._",
  ].join("\n");
  const labels = [
    "kind/security",
    "source/code-scanning",
    "status/needs-triage",
    appLabel(path),
    SEVERITY_PRIORITY[severity] ?? "priority/normal",
  ];
  return { title, body, labels };
}

/** Where each alert source is read from and how its issues are tagged. */
export const SOURCES = {
  dependabot: {
    name: "Dependabot alert",
    alerts: "/dependabot/alerts?state=open",
    alert: (n) => `/dependabot/alerts/${n}`,
    label: "source/dependabot",
    marker: /<!-- dependabot-alert:(\d+) -->/,
    issueFor,
  },
  "code-scanning": {
    name: "Code scanning alert",
    alerts: "/code-scanning/alerts?state=open",
    alert: (n) => `/code-scanning/alerts/${n}`,
    label: "source/code-scanning",
    marker: /<!-- code-scanning-alert:(\d+) -->/,
    issueFor: codeScanningIssueFor,
  },
};

async function main() {
  const { GH_TOKEN, GITHUB_REPOSITORY } = process.env;
  const dryRun = process.env.DRY_RUN === "true";
  const sourceArg = process.argv.indexOf("--source");
  const sourceName = sourceArg === -1 ? "dependabot" : process.argv[sourceArg + 1];
  const source = SOURCES[sourceName];
  if (!source) throw new Error(`unknown --source ${sourceName}; expected ${Object.keys(SOURCES).join(" or ")}`);
  if (!GH_TOKEN || !GITHUB_REPOSITORY) throw new Error("GH_TOKEN and GITHUB_REPOSITORY are required");

  const base = `https://api.github.com/repos/${GITHUB_REPOSITORY}`;
  const request = async (url, init = {}) => {
    const res = await fetch(url, {
      ...init,
      headers: {
        authorization: `Bearer ${GH_TOKEN}`,
        accept: "application/vnd.github+json",
        "user-agent": "alerts-to-issues",
      },
    });
    if (!res.ok) throw new Error(`${init.method ?? "GET"} ${url}: ${res.status} ${await res.text()}`);
    return res;
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

  const alerts = await all(source.alerts);
  const issues = (await all(`/issues?state=all&labels=${source.label},kind/security`)).filter((i) => !i.pull_request);
  const byAlert = new Map();
  for (const issue of issues) {
    const m = issue.body?.match(source.marker);
    if (m) byAlert.set(Number(m[1]), issue);
  }

  let created = 0;
  let reopened = 0;
  let closed = 0;
  const open = new Set(alerts.map((a) => a.number));
  for (const alert of alerts) {
    const existing = byAlert.get(alert.number);
    const issue = source.issueFor(alert);
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
    const alert = await api(source.alert(number));
    const reason = alert.state === "fixed" ? "completed" : "not_planned";
    console.log(`close #${issue.number}: alert ${number} is ${alert.state}`);
    if (!dryRun) {
      await api(`/issues/${issue.number}/comments`, {
        method: "POST",
        body: JSON.stringify({ body: `${source.name} ${number} is now **${alert.state}**; closing.` }),
      });
      await api(`/issues/${issue.number}`, { method: "PATCH", body: JSON.stringify({ state: "closed", state_reason: reason }) });
    }
    closed++;
  }
  console.log(
    `${alerts.length} open ${source.name}s: ${created} issues created, ${reopened} reopened, ${closed} closed${dryRun ? " (dry run)" : ""}.`,
  );
}

if (import.meta.url === `file://${process.argv[1].replaceAll("\\", "/").replace(/^(?=[A-Za-z]:)/, "/")}`) {
  await main();
}
