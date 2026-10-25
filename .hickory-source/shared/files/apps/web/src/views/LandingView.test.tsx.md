<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/LandingView.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Visit Is Reported Exactly Once Per Page Load](../../../../../../guarantees/analytics/landing-events-fire-once/contract.md)
- [The homepage opens with a live, paused debugger](../../../../../../guarantees/landing/homepage-opens-paused/contract.md)

Captured SHA-256: `7a20c8e0ad7e9879df1b684c89160a8fd5e6a2b90bfbae30a20030d5ba65798f`.

<book:entry path="apps/web/src/views/LandingView.test.tsx">
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
    expect(container.querySelectorAll("section")).toHaveLength(2);
    expect(screen.getByRole("link", { name: "Inspect the agent workflow" }).getAttribute("href")).toBe("#watch-workflow");
    expect(screen.getByRole("link", { name: "Current desktop prerelease downloads" }).getAttribute("href")).toContain("/releases/tag/unstable");
    expect(screen.queryByText("Public downloads coming soon.")).toBeNull();
    expect(screen.queryByText("Three things that should have been one")).toBeNull();
  });
  it("reports the visit once per load", () => {
    const { rerender } = render(<LandingView />); rerender(<LandingView />);
    expect(captured.filter((event) => event === "landing_viewed")).toHaveLength(1);
  });
});
</book:entry>

</book:doc>
