// status-worker: CI status for the dashboard and apps
import { test } from "node:test";
import assert from "node:assert/strict";
import worker from "../src/index.js";

const get = (path) => new Request(`https://status.example${path}`);
const fakeFetch =
  (body, status = 200) =>
  async () =>
    new Response(JSON.stringify(body), { status });

test("health returns ok", async () => {
  const res = await worker.fetch(get("/health"));
  assert.equal(res.status, 200);
  assert.deepEqual(await res.json(), { ok: true });
});

test("status reports the latest run", async () => {
  const run = { name: "CI", status: "completed", conclusion: "success", html_url: "https://x/1" };
  const res = await worker.fetch(get("/status"), { REPOSITORY: "o/r" }, null, fakeFetch({ workflow_runs: [run] }));
  assert.deepEqual(await res.json(), {
    repository: "o/r",
    latestRun: { name: "CI", status: "completed", conclusion: "success", url: "https://x/1" },
  });
});

test("status surfaces GitHub API errors as 502", async () => {
  const res = await worker.fetch(get("/status"), { REPOSITORY: "o/r" }, null, fakeFetch({}, 403));
  assert.equal(res.status, 502);
});

test("status without configuration is a 500", async () => {
  assert.equal((await worker.fetch(get("/status"))).status, 500);
});

test("unknown paths and methods are rejected", async () => {
  assert.equal((await worker.fetch(get("/nope"))).status, 404);
  assert.equal((await worker.fetch(new Request("https://s/health", { method: "POST" }))).status, 405);
});
