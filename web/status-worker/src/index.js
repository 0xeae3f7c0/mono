// status-worker: CI status for the dashboard and apps
// Serves a small JSON status document for the repository.
//   GET /health  -> {"ok": true}
//   GET /status  -> latest CI run for the default branch (via the GitHub API)

const json = (body, status = 200) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json; charset=utf-8" },
  });

export async function latestRun(repository, fetchImpl = fetch) {
  const res = await fetchImpl(`https://api.github.com/repos/${repository}/actions/runs?per_page=1&branch=main`, {
    headers: { "user-agent": "commit-status-worker", accept: "application/vnd.github+json" },
  });
  if (!res.ok) throw new Error(`GitHub API returned ${res.status}`);
  const [run] = (await res.json()).workflow_runs ?? [];
  return run ? { name: run.name, status: run.status, conclusion: run.conclusion, url: run.html_url } : null;
}

export default {
  async fetch(request, env = {}, _ctx, fetchImpl = fetch) {
    const { pathname } = new URL(request.url);
    if (request.method !== "GET") return json({ error: "method not allowed" }, 405);
    if (pathname === "/health") return json({ ok: true });
    if (pathname === "/status") {
      if (!env.REPOSITORY) return json({ error: "REPOSITORY is not configured" }, 500);
      try {
        return json({ repository: env.REPOSITORY, latestRun: await latestRun(env.REPOSITORY, fetchImpl) });
      } catch (error) {
        return json({ error: error.message }, 502);
      }
    }
    return json({ error: "not found" }, 404);
  },
};
