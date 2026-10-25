<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/visibleElementRect.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Ribbon Crosses Documents, Because Lineage Does](../../../../../contract.md)

Captured SHA-256: `eee6734ed6158e6f08e639465ccee2bd756b754a15652443192a4f4c279aaa9a`.

<book:entry path="apps/web/src/lib/visibleElementRect.test.ts">
// @vitest-environment jsdom
// Guarantee: docs/guarantees/lineage/a-ribbon-crosses-documents.md
import { afterEach, expect, it, vi } from "vitest";
import { findTerminal, attachTerminal } from "../shell/Ribbons";

afterEach(() => { document.body.replaceChildren(); vi.restoreAllMocks(); });
function fixture() {
  const container = document.createElement("div");
  container.innerHTML = '<div class="shell-pane__body"><div class="cm-scroller"><div data-tree-path="long-file.md"></div></div></div><button data-ribbon-port="file:long-file.md"></button>';
  document.body.append(container);
  const pane = container.querySelector<HTMLElement>(".shell-pane__body")!;
  const scroller = container.querySelector<HTMLElement>(".cm-scroller")!;
  const row = container.querySelector<HTMLElement>("[data-tree-path]")!;
  pane.style.overflow = scroller.style.overflow = "auto";
  vi.spyOn(pane, "getBoundingClientRect").mockReturnValue(new DOMRect(20, 10, 180, 300));
  vi.spyOn(scroller, "getBoundingClientRect").mockReturnValue(new DOMRect(30, 40, 160, 200));
  for (const [element, width, height] of [[pane, 180, 300], [scroller, 145, 185]] as const) {
    Object.defineProperties(element, { clientWidth: { value: width }, clientHeight: { value: height } });
  }
  const rect = vi.spyOn(row, "getBoundingClientRect").mockReturnValue(new DOMRect(40, 60, 600, 25));
  return { container, rect };
}
it.each([true, false])("lands on the visible row edge, excluding its scrollbar, braces=%s", braces => {
  const { container, rect } = fixture();
  const terminal = findTerminal(container, "file", "long-file.md")!;
  expect(terminal.ends).toBe("tree");
  expect(terminal.rect.right).toBe(175); // 30 + 145; the row's unclipped end is 640.
  const attach = attachTerminal(terminal, new DOMRect(), 800, { yTop: 50, yBot: 100 }, braces, { x: 790, y: 75 }, -1);
  expect(attach.connector).toMatchObject({ x: 172, width: 3 });
  expect(braces ? attach.link : attach.band).toContain("175");
  // Horizontal scrolling moves the row, but the visible right edge stays put.
  rect.mockReturnValue(new DOMRect(-200, 60, 600, 25));
  expect(findTerminal(container, "file", "long-file.md")!.rect.left).toBe(30);
  expect(findTerminal(container, "file", "long-file.md")!.rect.right).toBe(175);
});
it.each([new DOMRect(250, 60, 100, 25), new DOMRect(40, 230, 600, 25)])("uses the port when the row is fully clipped", hidden => {
  const { container, rect } = fixture(); rect.mockReturnValue(hidden);
  expect(findTerminal(container, "file", "long-file.md")!.ends).toBe("port");
});
</book:entry>

</book:doc>
