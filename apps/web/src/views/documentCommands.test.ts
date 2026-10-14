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
