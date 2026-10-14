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
