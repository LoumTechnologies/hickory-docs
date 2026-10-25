<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/mock/mockApi.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `409a121f71f8ebe17c193d77791a1344ade7f8a473d1ffe750b38b1c5b95af33`.

<book:entry path="apps/web/src/mock/mockApi.test.ts">
import { describe, expect, it } from "vitest";
import { installMockApi } from "./mockApi";
import { CLI_BLOCKS, CLI_SOURCE, PAPER_BLOCKS, PAPER_SOURCE } from "./mockData";

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
  });
});
</book:entry>

</book:doc>
