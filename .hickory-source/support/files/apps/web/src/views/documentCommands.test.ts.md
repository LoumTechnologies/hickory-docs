<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/documentCommands.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `8bc92c65fbee98b04c8d4fcec697a20671366a7520e06f4b0e4180bbc39776f7`.

<book:entry path="apps/web/src/views/documentCommands.test.ts">
import { expect, it, vi } from "vitest";
import { documentCommands } from "./documentCommands";
import type { DocSession } from "./documentSession";

it("uses the owning session for Run and Test, and names its target", () => {
  const session = { doc: { path: "project.md" }, runAll: vi.fn(), verify: vi.fn(), runningCells: new Set(), banner: null } as unknown as DocSession;
  const commands = documentCommands(session);
  expect(commands.map(item => [item.label, item.detail])).toEqual([["Run document", "project.md"], ["Test document", "project.md"]]);
  commands[0].run(); commands[1].run();
  expect(session.runAll).toHaveBeenCalledOnce(); expect(session.verify).toHaveBeenCalledOnce();
  expect(documentCommands({ ...session, runningCells: new Set(["cell"]) })).toEqual([]);
  expect(documentCommands({ ...session, banner: { kind: "pending", text: "Testing" } })).toEqual([]);
  expect(documentCommands(null)).toEqual([]);
});
</book:entry>

</book:doc>
