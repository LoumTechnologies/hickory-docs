import { describe, expect, it } from "vitest";
import { installMockApi } from "./mockApi";
import { CLI_BLOCKS, CLI_SOURCE, PAPER_BLOCKS, PAPER_SOURCE } from "./mockData";

// localStorage shim for setToken in node environment.
// @vitest-environment jsdom

describe("mock API", () => {
  it("mock block spans point into their source", () => {
    for (const [blocks, source] of [
      [CLI_BLOCKS, CLI_SOURCE],
      [PAPER_BLOCKS, PAPER_SOURCE],
    ] as const) {
      for (const b of blocks) {
        expect(b.span[0]).toBeGreaterThanOrEqual(0);
        expect(b.span[1]).toBeGreaterThan(b.span[0]);
        expect(b.span[1]).toBeLessThanOrEqual(source.length);
      }
    }
  });

  it("serves the contract routes through the typed client", async () => {
    const client = await import("../api/client");
    installMockApi();
    const projects = await client.api.projects();
    expect(projects.length).toBeGreaterThanOrEqual(2);
    const docs = await client.api.projectDocs(projects[0].id);
    expect(docs.length).toBeGreaterThan(0);
    const render = await client.api.render(docs[0].id);
    expect(render.blocks.some((b) => b.kind === "exec")).toBe(true);
    const plans = await client.api.plans();
    expect(plans.plans.length).toBeGreaterThan(0);
    expect(
      plans.plans.every((p) =>
        p.prices.every((price) => typeof price.amount_cents === "number"),
      ),
    ).toBe(true);
    // Mirrors plans.json's default plan set.
    expect(plans.plans.map((p) => p.key)).toEqual(["open", "pro", "team", "business"]);
  });
});
