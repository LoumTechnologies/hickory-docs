<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/WorkflowTour.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The engineer tour distinguishes replay from execution](../../../../../contract.md)

Captured SHA-256: `8e0f8634f01ce2c3d74c1846c4a4a4437c105b020cd896fa2e54edd04fbed63c`.

<book:entry path="apps/web/src/components/WorkflowTour.test.tsx">
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
vi.mock("../embed/DebuggableDocument", () => ({ DebuggableDocument: ({ revision, onExecutionResult }: {revision:string;onExecutionResult:(r:{revision:string;exitCode:number})=>void}) => <button onClick={() => onExecutionResult({ revision, exitCode: revision === "original-example" ? 1 : 0 })}>Run fixture</button> }));
import { WorkflowTour } from "./WorkflowTour";
afterEach(() => { cleanup(); localStorage.clear(); });
// Guarantee: docs/guarantees/authoring/the-engineer-tour-distinguishes-replay-from-execution.md
it("requires observed failure and success rather than only clicking Next", () => {
  render(<WorkflowTour surface="site" />);
  fireEvent.click(screen.getByRole("button", { name: "AI engineer tour" }));
  fireEvent.click(screen.getByText("Continue")); fireEvent.click(screen.getByText("Continue"));
  expect((screen.getByText("Continue") as HTMLButtonElement).disabled).toBe(true);
  fireEvent.change(screen.getByLabelText("Your objection"), { target: { value: "Retries exceed the budget" } });
  fireEvent.click(screen.getByText("Continue"));
  expect((screen.getByText("Continue") as HTMLButtonElement).disabled).toBe(true);
  fireEvent.click(screen.getByText("Run fixture")); fireEvent.click(screen.getByText("Continue"));
  expect((screen.getByText("Continue") as HTMLButtonElement).disabled).toBe(true);
  fireEvent.click(screen.getByText("Run fixture")); fireEvent.click(screen.getByText("Continue"));
  expect(screen.getByText("Review and retain evidence")).toBeTruthy();
});
</book:entry>

</book:doc>
