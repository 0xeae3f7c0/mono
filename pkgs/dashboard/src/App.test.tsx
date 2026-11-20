// dashboard: Main web UI (with Storybook) (commit platform, pkgs/dashboard)
// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import App from "./App";

describe("App", () => {
  it("increments the counter on click", () => {
    render(<App />);
    const button = screen.getByRole("button", { name: /count is 0/i });
    fireEvent.click(button);
    expect(screen.getByRole("button", { name: /count is 1/i })).toBeTruthy();
  });
});
