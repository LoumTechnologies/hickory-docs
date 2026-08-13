// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { setDeliveryForTest } from "../../analytics/sink";
import { AgentToolSurface } from "../../components/AgentToolSurface";
import { ProgramDemo } from "./ProgramDemo";

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

describe("the home page's demo", () => {
  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("weaves the document's files without being asked", () => {
    render(<ProgramDemo />);
    // The point of the page is on screen before the visitor clicks anything:
    // one document, two files woven from it.
    expect(screen.getByRole("button", { name: /^bisect\.py/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /^test_bisect\.py/ })).toBeTruthy();
  });

  // Protects docs/guarantees/landing/home-demos-run-the-real-mechanism.md
  it("shows no transcript until the cells are run, and names it a recording", () => {
    render(<ProgramDemo />);
    expect(screen.queryByLabelText("Captured run")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: /run the cells/i }));
    const run = screen.getByLabelText("Captured run");
    expect(run.textContent).toContain("3");
    // The one simulated thing on the page says so, next to itself.
    expect(run.textContent).toMatch(/recorded transcript/i);
  });

  it("records engagement once, naming the demo and what was done", () => {
    render(<ProgramDemo />);
    fireEvent.click(screen.getByRole("button", { name: /run the cells/i }));
    fireEvent.click(screen.getByRole("button", { name: /run the cells again/i }));

    const engagements = captured.filter((c) => c.event === "demo_engaged");
    expect(engagements).toHaveLength(1);
    expect(engagements[0].properties.demo_id).toBe("program");
    expect(engagements[0].properties.step).toBe("run-cells");
  });
});

describe("the agent tool surface", () => {
  // Protects docs/guarantees/landing/home-page-claims-are-true-of-the-binary.md
  it("lists only commands the binary actually has", () => {
    render(<AgentToolSurface />);
    const text = document.body.textContent ?? "";
    // Every one of these is a real subcommand in crates/hickory-cli/src/main.rs.
    for (const command of [
      "hick init",
      "hick mcp",
      "hick doc read",
      "hick doc read-output",
      "hick doc edit-output",
      "hick doc edit",
      "hick doc verify",
      "hick promote",
    ]) {
      expect(text).toContain(command);
    }
  });

  // Protects docs/guarantees/landing/home-page-claims-are-true-of-the-binary.md
  it("promises no account, no server, and nothing to buy", () => {
    render(<AgentToolSurface />);
    const text = (document.body.textContent ?? "").toLowerCase();
    // The words a page for a local-only tool must never contain — every one
    // of them was on this page while it was pitching a hosted workspace.
    for (const word of ["sign up", "pricing", "per seat", "free trial", "upgrade"]) {
      expect(text).not.toContain(word);
    }
  });
});
