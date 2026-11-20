import { describe, expect, it } from "vitest";
import { app } from "../src/index";

/** In-memory stand-in for the KV binding. */
function memoryKv(): KVNamespace {
  const data = new Map<string, string>();
  return {
    get: async (k: string) => data.get(k) ?? null,
    put: async (k: string, v: string) => void data.set(k, v),
    delete: async (k: string) => void data.delete(k),
  } as unknown as KVNamespace;
}

describe("api-worker", () => {
  const env = { STORE: memoryKv() };
  const req = (method: string, path: string, json?: unknown) =>
    app.request(
      path,
      json === undefined
        ? { method }
        : { method, body: JSON.stringify(json), headers: { "content-type": "application/json" } },
      env,
    );

  it("stores, reads and deletes values", async () => {
    expect((await req("PUT", "/kv/greeting", { value: "hi" })).status).toBe(204);
    expect(await (await req("GET", "/kv/greeting")).json()).toEqual({ key: "greeting", value: "hi" });
    expect((await req("DELETE", "/kv/greeting")).status).toBe(204);
    expect((await req("GET", "/kv/greeting")).status).toBe(404);
  });

  it("validates keys and bodies", async () => {
    expect((await req("GET", "/kv/Bad_Key")).status).toBe(400);
    expect((await req("PUT", "/kv/ok", { value: 5 })).status).toBe(400);
  });
});
