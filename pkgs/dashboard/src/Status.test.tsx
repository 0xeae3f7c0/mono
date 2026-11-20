// @vitest-environment jsdom
import { afterAll, afterEach, beforeAll, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";
import { Status } from "./Status";

const server = setupServer();
beforeAll(() => server.listen());
afterEach(() => {
  server.resetHandlers();
  cleanup();
});
afterAll(() => server.close());

function renderStatus() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
      <Status />
    </QueryClientProvider>,
  );
}

describe("Status", () => {
  it("shows the latest run", async () => {
    server.use(
      http.get("*/status", () =>
        HttpResponse.json({
          repository: "o/r",
          latestRun: { name: "CI", status: "completed", conclusion: "success", url: "https://x/1" },
        }),
      ),
    );
    renderStatus();
    expect(await screen.findByText(/is success/)).toBeTruthy();
    expect(screen.getByRole("link", { name: "CI" }).getAttribute("href")).toBe("https://x/1");
  });

  it("reports worker errors", async () => {
    server.use(http.get("*/status", () => new HttpResponse(null, { status: 502 })));
    renderStatus();
    expect((await screen.findByRole("alert")).textContent).toContain("returned 502");
  });
});
