import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { setDeliveryForTest } from "../analytics/sink";
import { DECLARED_SEGMENTS, INTERESTS } from "../landing/interests";
import { LandingView, resetViewedForTest } from "./LandingView";

type Captured = { event: string; properties: Record<string, unknown> };

let captured: Captured[] = [];

beforeEach(() => {
  localStorage.clear();
  captured = [];
  resetViewedForTest();
  setDeliveryForTest((event, body) => {
    captured.push({
      event,
      properties: (body as { properties: Record<string, unknown> }).properties,
    });
  });
});

// This project runs vitest without `globals`, so Testing Library never
// registers its automatic afterEach cleanup and renders would otherwise stack
// up across tests.
afterEach(cleanup);

const eventsNamed = (name: string) => captured.filter((c) => c.event === name);

describe("the discovery landing page", () => {
  // Protects docs/guarantees/landing/discovery-page-is-interest-organized.md
  it("titles every section by a job or pain, never by an audience label", () => {
    render(<LandingView />);
    // The failure this guards against is a section reading "For Platform
    // Engineers" — which re-imposes our guess about who cares and destroys
    // the discovery the page exists to do.
    for (const interest of INTERESTS) {
      expect(screen.getByRole("button", { name: new RegExp(interest.title, "i") })).toBeTruthy();
      expect(interest.title).not.toMatch(/^for\b/i);
    }
  });

  // Protects docs/guarantees/analytics/landing-events-fire-once.md
  it("reports the visit exactly once per page load", () => {
    const { rerender } = render(<LandingView />);
    rerender(<LandingView />);
    expect(eventsNamed("landing_viewed")).toHaveLength(1);
  });

  // Protects docs/guarantees/analytics/landing-interest-vector.md
  it("records which interest was opened, and the dwell when it closes", () => {
    render(<LandingView />);
    const first = INTERESTS[0];
    const toggle = screen.getByRole("button", { name: new RegExp(first.title, "i") });

    fireEvent.click(toggle);
    const opened = eventsNamed("interest_expanded");
    expect(opened).toHaveLength(1);
    expect(opened[0].properties).toMatchObject({
      interest_id: first.id,
      phase: "open",
      open_index: 0,
    });

    fireEvent.click(toggle);
    const closed = eventsNamed("interest_expanded");
    expect(closed).toHaveLength(2);
    expect(closed[1].properties.phase).toBe("close");
    expect(typeof closed[1].properties.dwell_ms).toBe("number");
  });

  // Protects docs/guarantees/analytics/landing-interest-vector.md
  it("reveals the body only once the section is opened", () => {
    render(<LandingView />);
    const first = INTERESTS[0];
    const body = document.getElementById(`interest-body-${first.id}`)!;
    expect(body.hasAttribute("hidden")).toBe(true);
    fireEvent.click(screen.getByRole("button", { name: new RegExp(first.title, "i") }));
    expect(body.hasAttribute("hidden")).toBe(false);
  });

  // Protects docs/guarantees/analytics/landing-declared-vs-revealed.md
  it("records the declared identity and carries it on later events", () => {
    render(<LandingView />);
    const segment = DECLARED_SEGMENTS[0];

    fireEvent.click(screen.getByRole("button", { name: segment.label }));
    const declared = eventsNamed("segment_declared");
    expect(declared).toHaveLength(1);
    // The event must carry the NEW claim, not the stale one read before it.
    expect(declared[0].properties.declared_segment).toBe(segment.id);

    // The primary call to action is installing the tool — there is no signup
    // to click now that the product is a program you run (local-first.md).
    fireEvent.click(screen.getAllByRole("button", { name: /copy install command/i })[0]);
    const cta = eventsNamed("cta_clicked");
    expect(cta[0].properties).toMatchObject({
      cta_id: "install",
      declared_segment: segment.id,
    });
  });

  // Protects docs/guarantees/analytics/landing-declared-vs-revealed.md
  it("asks for identity only once, and does not block anything on the answer", () => {
    render(<LandingView />);
    const segment = DECLARED_SEGMENTS[0];
    fireEvent.click(screen.getByRole("button", { name: segment.label }));
    expect(screen.queryByRole("button", { name: segment.label })).toBeNull();
    // The CTA was reachable before the question was ever answered.
    expect(
      screen.getAllByRole("button", { name: /copy install command/i }).length,
    ).toBeGreaterThan(0);
  });

  // Protects docs/guarantees/analytics/landing-declared-vs-revealed.md
  it("stamps intended segment as absent when no campaign named one", () => {
    render(<LandingView />);
    expect(eventsNamed("landing_viewed")[0].properties.intended_segment).toBe("$none");
  });
});
