// docs/guarantees/agent/acp-agents-are-first-class.md
import { afterEach, expect, it } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { FeatureSettings } from "./FeatureSettings";
afterEach(cleanup);
it("opens outside a clipped pane, keeps controls usable, and dismisses locally", () => {
  const { container } = render(<div style={{ overflow: "hidden" }}><FeatureSettings label="Agent settings"><label>Agent<select aria-label="Agent"><option>Codex</option></select></label></FeatureSettings></div>);
  const gear = screen.getByRole("button", { name: "Agent settings" });
  expect(screen.queryByRole("region")).toBeNull();
  fireEvent.click(gear);
  const panel = screen.getByRole("region", { name: "Agent settings" });
  expect(container.contains(panel)).toBe(false);
  fireEvent.pointerDown(screen.getByLabelText("Agent"));
  expect(screen.getByRole("region")).toBe(panel);
  fireEvent.keyDown(panel, { key: "Escape" });
  expect(screen.queryByRole("region")).toBeNull();
  fireEvent.click(gear);
  fireEvent.pointerDown(document.body);
  expect(screen.queryByRole("region")).toBeNull();
});
