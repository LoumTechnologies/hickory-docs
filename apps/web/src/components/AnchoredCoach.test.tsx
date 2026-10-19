// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { AnchoredCoach } from "./AnchoredCoach";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
it("preserves target focus and dismisses with Escape only inside its nonmodal popup", () => {
  const target = document.createElement("button"); target.id = "coach-target"; target.textContent = "Real control"; document.body.append(target);
  vi.spyOn(target, "getClientRects").mockReturnValue([{ x: 20, y: 20, width: 100, height: 30 }] as unknown as DOMRectList);
  vi.spyOn(target, "getBoundingClientRect").mockReturnValue({ left: 20, right: 120, top: 20, bottom: 50, width: 100, height: 30 } as DOMRect);
  target.scrollIntoView = vi.fn(); target.focus();
  const dismiss = vi.fn();
  try {
    render(<AnchoredCoach target="#coach-target" title="Investigation" onDismiss={dismiss}><p>Use the actual control</p></AnchoredCoach>);
    expect(document.activeElement).toBe(target);
    expect(screen.getByRole("region", { name: "Investigation" }).getAttribute("aria-modal")).toBeNull();
    fireEvent.keyDown(window, { key: "Escape" }); expect(dismiss).not.toHaveBeenCalled();
    const locate = screen.getByRole("button", { name: "Locate target" }); locate.focus(); fireEvent.click(locate);
    expect(target.scrollIntoView).toHaveBeenCalled(); expect(document.activeElement).toBe(target);
    screen.getByRole("button", { name: "Dismiss coach" }).focus(); fireEvent.keyDown(window, { key: "Escape" }); expect(dismiss).toHaveBeenCalledOnce();
  } finally { target.remove(); }
});
it("keeps recovery available when the normal target pane is absent", () => {
  render(<AnchoredCoach target="#closed-pane" title="Inspect evidence" onDismiss={() => {}}><button>Skip unestablished</button></AnchoredCoach>);
  expect(screen.getByRole("status").textContent).toContain("closed, hidden or unavailable");
  expect(screen.getByRole("button", { name: "Locate target" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "Skip unestablished" })).toBeTruthy();
});
