<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/landing/demos/demos.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e2fe96c33971"></a>
## `apps/web/src/landing/demos/demos.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Driving The Home-Page Demo Is Reported As Revealed Interest](../../../../../../guarantees/analytics/landing-demo-engagement/contract.md), [The Lineage Browser Shows One Column Per Stage](../../../../../../guarantees/authoring/the-lineage-browser-shows-one-column-per-stage/contract.md), [Every Claim On The Home Page Is True Of The Downloaded Binary](../../../../../../guarantees/landing/home-page-claims-are-true-of-the-binary/contract.md).

<a id="f-e2fe96c33971-Captured"></a>
### `Captured`

<storycode:copy id="f-e2fe96c33971-Captured">
// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { setDeliveryForTest } from "../../analytics/sink";
import { AgentToolSurface } from "../../components/AgentToolSurface";
import { ProgramDemo } from "./ProgramDemo";

type Captured = { event: string; properties: Record<string, unknown> };
</storycode:copy>
<a id="f-e2fe96c33971-captured"></a>
### `captured`

<storycode:copy id="f-e2fe96c33971-captured">
let captured: Captured[] = [];

</storycode:copy>
<a id="f-e2fe96c33971-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e2fe96c33971-expression_statement">
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

</storycode:copy>
<a id="f-e2fe96c33971-expression_statement-2"></a>
### `expression_statement`

vitest runs without `globals` here, so Testing Library never registers its
own cleanup and renders would stack across tests.

<storycode:copy id="f-e2fe96c33971-expression_statement-2">
// vitest runs without `globals` here, so Testing Library never registers its
// own cleanup and renders would stack across tests.
afterEach(cleanup);

</storycode:copy>
<a id="f-e2fe96c33971-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-e2fe96c33971-expression_statement-3">
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

</storycode:copy>
<a id="f-e2fe96c33971-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-e2fe96c33971-expression_statement-4">
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
      "hick ingest --from session",
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
</storycode:copy>

### Assemble `apps/web/src/landing/demos/demos.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/landing/demos/demos.test.tsx"><storycode:paste select="#f-e2fe96c33971-Captured"/><storycode:paste select="#f-e2fe96c33971-captured"/><storycode:paste select="#f-e2fe96c33971-expression_statement"/><storycode:paste select="#f-e2fe96c33971-expression_statement-2"/><storycode:paste select="#f-e2fe96c33971-expression_statement-3"/><storycode:paste select="#f-e2fe96c33971-expression_statement-4"/></storycode:file>

</storycode:doc>
