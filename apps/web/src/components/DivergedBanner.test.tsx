// Protects docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md
// and docs/specs/freeform/three-axes.md, axis 3.
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { DivergedBanner } from "./DivergedBanner";

afterEach(cleanup);

describe("the one diverged surface", () => {
  it("offers the same three ways out, and says what mine and theirs are", () => {
    const keep = vi.fn();
    const take = vi.fn();
    const merge = vi.fn();
    render(
      <DivergedBanner
        keepLabel="Leave disk unchanged" keepEffect="Dismisses the warning without writing."
        takeLabel="Regenerate from document" takeEffect="Replaces the disk file with the document output."
        what="client/client.py"
        reason="an edit to generated text has no source to carry back to"
        mine="what is on disk"
        theirs="what the document produces"
        onKeepMine={keep}
        onTakeTheirs={take}
        onMerge={merge}
      />,
    );
    expect(screen.getByRole("alert").textContent).toMatch(/what is on disk is not what the document produces/);
    fireEvent.click(screen.getByRole("button", { name: "Leave disk unchanged" }));
    fireEvent.click(screen.getByRole("button", { name: "Regenerate from document" }));
    fireEvent.click(screen.getByRole("button", { name: "Merge…" }));
    expect(keep).toHaveBeenCalled();
    expect(take).toHaveBeenCalled();
    expect(merge).toHaveBeenCalled();
  });

  it("says what would make theirs exist when there is nothing to take yet", () => {
    render(
      <DivergedBanner
        keepLabel="Leave disk unchanged" keepEffect="Dismisses the warning without writing."
        takeLabel="Regenerate from document" takeEffect="Replaces the disk file with the document output."
        what="report.md"
        reason="2 cell(s) are unrecorded"
        mine="what is on disk"
        theirs="what the document produces"
        onKeepMine={() => {}}
        takeTheirsHint="Run the document to have a version to take."
      />,
    );
    expect(screen.queryByRole("button", { name: "Regenerate from document" })).toBeNull();
    expect(screen.getByText(/Run the document/)).toBeTruthy();
  });
});
