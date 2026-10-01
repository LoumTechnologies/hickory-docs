// Opening an already-open file flashes its tab — the activation alone is
// invisible when the tab was already front-and-centre.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { attachFlashListener, FLASH_CLASS, flashTab } from "./flashTab";

function tab(root: HTMLElement, kind: string, target: string): HTMLElement {
  const el = document.createElement("button");
  el.setAttribute("data-shell-tab-kind", kind);
  el.setAttribute("data-shell-tab-target", target);
  root.appendChild(el);
  return el;
}

describe("flashTab", () => {
  let root: HTMLElement;
  let detach: () => void;

  beforeEach(() => {
    vi.stubGlobal("requestAnimationFrame", (cb: FrameRequestCallback) => {
      cb(0);
      return 1;
    });
    vi.stubGlobal("cancelAnimationFrame", () => undefined);
    root = document.createElement("div");
    document.body.appendChild(root);
    detach = attachFlashListener(root);
  });

  afterEach(() => {
    detach();
    root.remove();
    vi.unstubAllGlobals();
  });

  it("pulses the matching tab and clears the class when the animation ends", () => {
    const el = tab(root, "document", "notes.md");
    flashTab("document", "notes.md");
    expect(el.classList.contains(FLASH_CLASS)).toBe(true);
    el.dispatchEvent(new Event("animationend"));
    expect(el.classList.contains(FLASH_CLASS)).toBe(false);
  });

  it("matches kind AND target — a generated file's path never flashes a doc tab", () => {
    const doc = tab(root, "document", "orders.py");
    const gen = tab(root, "generated", "orders.py");
    flashTab("generated", "orders.py");
    expect(doc.classList.contains(FLASH_CLASS)).toBe(false);
    expect(gen.classList.contains(FLASH_CLASS)).toBe(true);
  });

  it("survives targets with quotes and backslashes", () => {
    const el = tab(root, "generated", 'we"ird\\name.py');
    flashTab("generated", 'we"ird\\name.py');
    expect(el.classList.contains(FLASH_CLASS)).toBe(true);
  });

  it("re-fires an in-flight flash from scratch", () => {
    const el = tab(root, "document", "a.md");
    flashTab("document", "a.md");
    flashTab("document", "a.md");
    expect(el.classList.contains(FLASH_CLASS)).toBe(true);
  });

  it("does nothing after detach", () => {
    const el = tab(root, "document", "a.md");
    detach();
    flashTab("document", "a.md");
    expect(el.classList.contains(FLASH_CLASS)).toBe(false);
    detach = () => undefined;
  });
});
