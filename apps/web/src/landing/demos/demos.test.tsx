// @vitest-environment jsdom
import { StrictMode } from "react";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { setDeliveryForTest } from "../../analytics/sink";
import { CollaborationDemo } from "./CollaborationDemo";
import { KnowledgeWorkDemo } from "./KnowledgeWorkDemo";
import { KNOWLEDGE_STEPS } from "./scripts";

type Captured = { event: string; properties: Record<string, unknown> };
let captured: Captured[] = [];

beforeEach(() => {
  localStorage.clear();
  captured = [];
  setDeliveryForTest((event, body) => {
    captured.push({
      event,
      properties: (body as { properties: Record<string, unknown> }).properties,
    });
  });
});

// vitest runs without `globals` here, so Testing Library never registers its
// own cleanup and renders would stack across tests.
afterEach(cleanup);

const step = (index: number) =>
  screen.getByRole("button", { name: new RegExp(KNOWLEDGE_STEPS[index].label, "i") });

describe("the knowledge-work walkthrough", () => {
  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("generates nothing until the agent has run, then one node per ticket", () => {
    render(<KnowledgeWorkDemo />);
    expect(screen.queryByRole("button", { name: /HD-412/ })).toBeNull();

    fireEvent.click(step(3));
    for (const key of ["HD-412", "HD-413", "HD-414"]) {
      expect(screen.getByRole("button", { name: new RegExp(key) })).toBeTruthy();
    }
  });

  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("shows the generated side as Jira's copy, and says it is simulated", () => {
    render(<KnowledgeWorkDemo />);
    fireEvent.click(step(3));
    expect(screen.getByText(/Jira · HD-412 \(simulated\)/)).toBeTruthy();
    // Claiming a live integration we do not have would be the one lie this
    // page cannot afford.
    expect(screen.getByText(/no issue tracker is contacted/i)).toBeTruthy();
  });

  // Protects docs/guarantees/analytics/landing-demo-engagement.md
  it("reports which step the visitor reached", () => {
    render(<KnowledgeWorkDemo />);
    fireEvent.click(step(1));
    const events = captured.filter((c) => c.event === "demo_engaged");
    expect(events).toHaveLength(1);
    expect(events[0].properties).toMatchObject({
      demo_id: "knowledge-work",
      step: KNOWLEDGE_STEPS[1].id,
    });
  });

  it("does not report a step the visitor was already on", () => {
    render(<KnowledgeWorkDemo />);
    fireEvent.click(step(0));
    expect(captured.filter((c) => c.event === "demo_engaged")).toHaveLength(0);
  });

  // Protects docs/guarantees/analytics/landing-demo-engagement.md
  it("separates a demo that was watched from one that was driven", () => {
    render(<KnowledgeWorkDemo />);
    fireEvent.click(step(1));
    const driven = captured.filter((c) => c.event === "demo_engaged");
    // Someone who pressed a step reached it themselves. Counting an autoplay
    // tick as the same act would inflate exactly the number that is supposed
    // to mean "this person tried the product".
    expect(driven[0].properties.autoplay).toBe(false);
  });

  it("offers to play itself, and hands control back at the hands-on step", () => {
    render(<KnowledgeWorkDemo />);
    const play = screen.getByRole("button", { name: /play it for me/i });
    fireEvent.click(play);
    expect(screen.getByRole("button", { name: /pause/i })).toBeTruthy();
    // Pressing any step must stop autoplay rather than fight the visitor.
    fireEvent.click(step(2));
    expect(screen.queryByRole("button", { name: /pause/i })).toBeNull();
  });
});

describe("the collaboration demo", () => {
  it("puts the same document in front of both people", () => {
    render(<CollaborationDemo />);
    // Two independent clients, each with its own editor over one document.
    expect(screen.getByTestId("demo-collab-a-source")).toBeTruthy();
    expect(screen.getByTestId("demo-collab-b-source")).toBeTruthy();
  });

  it("has nothing to commit or push until something changes", () => {
    render(<CollaborationDemo />);
    expect(screen.getByRole("button", { name: /nothing to commit/i }).hasAttribute("disabled")).toBe(
      true,
    );
    expect(screen.getByRole("button", { name: /^push$/i }).hasAttribute("disabled")).toBe(true);
  });

  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  //
  // Under StrictMode on purpose: that is how the app actually mounts, and the
  // first version of this demo built its room in a `useMemo`, whose cleanup
  // from StrictMode's throwaway first mount tore down the relay the surviving
  // editors were bound to. Both panes still rendered; they just silently
  // stopped seeing each other, which no non-StrictMode test could catch.
  it("lands a pulled commit in both panes at once, even under StrictMode", () => {
    render(
      <StrictMode>
        <CollaborationDemo />
      </StrictMode>,
    );
    fireEvent.click(screen.getByRole("button", { name: /^pull$/i }));
    expect(screen.getByText(/Add the enterprise row/)).toBeTruthy();
    for (const id of ["demo-collab-a-source", "demo-collab-b-source"]) {
      expect(screen.getByTestId(id).textContent).toContain("enterprise");
    }
  });
});
