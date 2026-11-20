// Cloudflare Worker API built with Hono: a validated key-value store in KV.
//   GET    /kv/:key         -> {"key", "value"} or 404
//   PUT    /kv/:key {value} -> 204
//   DELETE /kv/:key         -> 204
import { Hono } from "hono";
import { zValidator } from "@hono/zod-validator";
import { z } from "zod";

export type Env = { Bindings: { STORE: KVNamespace } };

const key = z.object({ key: z.string().regex(/^[a-z0-9-]{1,64}$/) });
const body = z.object({ value: z.string().max(1024) });

export const app = new Hono<Env>()
  .get("/health", (c) => c.json({ ok: true }))
  .get("/kv/:key", zValidator("param", key), async (c) => {
    const { key } = c.req.valid("param");
    const value = await c.env.STORE.get(key);
    return value === null ? c.json({ error: "not found" }, 404) : c.json({ key, value });
  })
  .put("/kv/:key", zValidator("param", key), zValidator("json", body), async (c) => {
    await c.env.STORE.put(c.req.valid("param").key, c.req.valid("json").value);
    return c.body(null, 204);
  })
  .delete("/kv/:key", zValidator("param", key), async (c) => {
    await c.env.STORE.delete(c.req.valid("param").key);
    return c.body(null, 204);
  });

export default app;
