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
