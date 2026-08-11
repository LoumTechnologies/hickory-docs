import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { setDeliveryForTest } from "../analytics/sink";
import { LandingView, resetViewedForTest } from "./LandingView";
import { PricingView } from "./PricingView";

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
    // The demos are simulations that run in the browser; the page must not
    // reach for an API that a static deployment does not have.
    expect(fetchSpy).not.toHaveBeenCalled();
    fetchSpy.mockRestore();
  });

  it("offers installing the tool, not signing up for a workspace", () => {
    const { container } = render(<LandingView />);
    expect(screen.getAllByRole("button", { name: /copy install command/i }).length)
      .toBeGreaterThan(0);
    expect(container.textContent).toContain("curl");
    // The hosted front door is hidden unless this build is the hosted app.
    expect(container.textContent).not.toContain("Start free");
    expect(container.textContent).not.toContain("hosted workspace");
  });

  it("prices from the generated catalogue, with no request and no checkout", () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch");
    const { container } = render(<PricingView />);

    // Plans come from plans.json through the server's own projection, so the
    // page is never in a loading state and never wrong about pricing.
    expect(container.textContent).not.toContain("Loading plans");
    expect(screen.getByRole("heading", { name: "Open" })).toBeTruthy();
    expect(fetchSpy).not.toHaveBeenCalled();

    // No subscribe buttons: there is no billing service on a static site, and
    // a button that cannot work is worse than no button.
    expect(container.textContent).not.toContain("Start 14-day trial");
    // …and the page says plainly what is free instead.
    expect(container.textContent).toContain("free and needs no");
    fetchSpy.mockRestore();
  });
});
