<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/StaticSite.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Every Claim On The Home Page Is True Of The Downloaded Binary](../../../../../contract.md)

Captured SHA-256: `eb4f2ee2a53069b127d69d840b09eacf97769c3d52054528f49e53136197b8b4`.

<book:entry path="apps/web/src/views/StaticSite.test.tsx">
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { setDeliveryForTest } from "../analytics/sink";
import { LandingView, resetViewedForTest } from "./LandingView";

/**
 * The marketing site is a static file: no API, no accounts, no billing.
 *
 * These tests protect that property from the direction it will actually be
 * broken — someone adding a fetch to the landing path, or a subscribe button to
 * a page with nothing behind it. See docs/specs/freeform/local-first.md.
 */

beforeEach(() => {
  localStorage.clear();
  resetViewedForTest();
  setDeliveryForTest(() => {});
});

afterEach(cleanup);

describe("the marketing site with no server behind it", () => {
  it("renders the landing page without making a single request", () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch");
    render(<LandingView />);
    expect(screen.getByRole("heading", { level: 1 })).toBeTruthy();
    // Rendering mounts no API client. Asset loading and the real worker
    // debugger are exercised by the static Playwright acceptance suite.
    expect(fetchSpy).not.toHaveBeenCalled();
    fetchSpy.mockRestore();
  });

  it("offers installing the tool, not signing up for a workspace", () => {
    const { container } = render(<LandingView />);
    expect(screen.getByRole("link", { name: "Get the desktop app" })).toBeTruthy();
    expect(container.textContent).toContain("Downloadable software");
    // The hosted front door is hidden unless this build is the hosted app.
    expect(container.textContent).not.toContain("Start free");
    expect(container.textContent).not.toContain("hosted workspace");
  });

});
</book:entry>

</book:doc>
