import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { setDeliveryForTest } from "../analytics/sink";
import { LandingView, resetViewedForTest } from "./LandingView";

// Browser execution is exercised in all three real engines by Playwright.
vi.mock("../landing/demos/BrowserDebugDemo", () => ({ BrowserDebugDemo: () => <section aria-label="Live document demo" /> }));
let captured: string[] = [];
beforeEach(() => {
  localStorage.clear(); captured = []; resetViewedForTest();
  setDeliveryForTest((event) => { captured.push(event); });
});
afterEach(cleanup);
describe("the short homepage", () => {
  it("identifies downloadable software before the live example", () => {
    const { container } = render(<LandingView />);
    const header = container.querySelector("header")!;
    expect(header.textContent).toContain("Downloadable software");
    expect(header.textContent).toContain("Runs on your machine");
    expect(screen.getByRole("link", { name: "Get the desktop app" }).getAttribute("href")).toBe("#download");
    expect(container.querySelectorAll("section")).toHaveLength(1);
    expect(screen.queryByText("Three things that should have been one")).toBeNull();
  });
  it("reports the visit once per load", () => {
    const { rerender } = render(<LandingView />); rerender(<LandingView />);
    expect(captured.filter((event) => event === "landing_viewed")).toHaveLength(1);
  });
});
