// A ribbon yields its click to any precise control beneath it — the ribbon
// can be hit anywhere, the button underneath only exactly where it is.
import { describe, expect, it } from "vitest";

import { preciseTargetInStack } from "./ribbonClickThrough";

function build(html: string): HTMLElement {
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.appendChild(host);
  return host;
}

describe("preciseTargetInStack", () => {
  it("yields to a button under the ribbon layer", () => {
    const host = build(
      `<svg class="ribbon-layer"><g><path /></g></svg>
       <button class="tab-close">×</button>`,
    );
    const layer = host.querySelector("svg")!;
    const path = host.querySelector("path")!;
    const button = host.querySelector("button")!;
    expect(preciseTargetInStack([path, layer, button, host], layer)).toBe(button);
  });

  it("resolves a click landing inside a control to the control itself", () => {
    const host = build(`<button><span class="icon">▸</span></button>`);
    const span = host.querySelector("span")!;
    expect(preciseTargetInStack([span, host], null)).toBe(host.querySelector("button"));
  });

  it("keeps the click when only plain content is beneath", () => {
    // Code text: the caret is one un-ribboned pixel away, the ribbon is the
    // more plausible intent — the ribbon acts.
    const host = build(`<div class="cm-line">let x = 1;</div>`);
    const line = host.querySelector(".cm-line")!;
    expect(preciseTargetInStack([line, host], null)).toBeNull();
  });

  it("only consults the topmost non-ribbon element", () => {
    // The button here is BEHIND an opaque pane: it was never clickable, so
    // the ribbon must not resurrect it.
    const host = build(
      `<div class="pane"><div class="cm-line">text</div></div><button>hidden</button>`,
    );
    const line = host.querySelector(".cm-line")!;
    const button = host.querySelector("button")!;
    expect(preciseTargetInStack([line, host, button], null)).toBeNull();
  });

  it("recognises tabs, ports, and gutter elements as precise", () => {
    const host = build(
      `<div data-shell-tab-kind="generated">tab</div>
       <div data-ribbon-port="generated:a.py">port</div>
       <div class="cm-gutterElement">12</div>`,
    );
    for (const el of Array.from(host.children)) {
      expect(preciseTargetInStack([el], null)).toBe(el);
    }
  });
});
