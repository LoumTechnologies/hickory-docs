<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/PaneHeader.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The editor keeps its space](../../../../../contract.md)

Captured SHA-256: `4a9185b0547ccf0c7b78a1ca5fe6f07040d8de224035b629aeb46470cbd5abc2`.

<book:entry path="apps/web/src/shell/PaneHeader.test.tsx">
import { afterEach, expect, it } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { PaneHeaderProvider, PaneHeaderSlots, PaneHeaderContent } from "./PaneHeader";
import { SurfaceInfo } from "../components/SurfaceInfo";

afterEach(cleanup);
// Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
it("keeps info and actions in the header without recreating the editing surface", () => {
  function Pane() {
    const [label, setLabel] = useState("old");
    return <PaneHeaderProvider><header data-testid="header"><PaneHeaderSlots /></header><main>
      <SurfaceInfo name="Note" editable destination="Saves to note.md" />
      <PaneHeaderContent part="actions"><button onClick={() => setLabel("new")}>{label}</button></PaneHeaderContent>
      <textarea aria-label="Source" defaultValue="unchanged" />
    </main></PaneHeaderProvider>;
  }
  render(<Pane />);
  const source = screen.getByRole("textbox") as HTMLTextAreaElement;
  source.setSelectionRange(1, 4);
  expect(screen.getByTestId("header").contains(screen.getByRole("button", { name: "old" }))).toBe(true);
  expect(screen.getByTestId("header").querySelector(".surface-info")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "old" }));
  expect(screen.getByRole("textbox")).toBe(source);
  expect(source.value).toBe("unchanged");
  expect([source.selectionStart, source.selectionEnd]).toEqual([1, 4]);
});
</book:entry>

</book:doc>
