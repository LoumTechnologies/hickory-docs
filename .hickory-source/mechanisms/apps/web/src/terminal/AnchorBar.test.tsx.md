<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/terminal/AnchorBar.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-635790b9cca6"></a>
## `apps/web/src/terminal/AnchorBar.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-635790b9cca6-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-635790b9cca6-expression_statement">
// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AnchorBar } from "./AnchorBar";

afterEach(cleanup);

</storycode:copy>
<a id="f-635790b9cca6-anchored"></a>
### `anchored`

<storycode:copy id="f-635790b9cca6-anchored">
const anchored = {
  doc: "abc123",
  container: "sdk",
  suspended: null,
  foreign: null,
  recorded: 3,
};

</storycode:copy>
<a id="f-635790b9cca6-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-635790b9cca6-expression_statement-2">
describe("what an anchored terminal says about itself", () => {
  it("says nothing at all when the terminal writes nothing", () => {
    // The ephemeral binding is the common case. Chrome reading "not
    // recording" on every scratch shell would train people to stop reading
    // this strip, which is the one thing it cannot afford.
    const { container } = render(
      <AnchorBar anchor={null} onResume={() => {}} onUnanchor={() => {}} />,
    );
    expect(container.firstChild).toBeNull();
  });

  it("names the document and the container, because that is the anchor", () => {
    render(
      <AnchorBar
        anchor={anchored}
        docName="scaffolding.md"
        onResume={() => {}}
        onUnanchor={() => {}}
      />,
    );
    const bar = screen.getByTestId("anchor-bar");
    expect(bar.textContent).toContain("writing");
    expect(bar.textContent).toContain("scaffolding.md");
    // The container is the anchor itself, not decoration.
    expect(bar.textContent).toContain("sdk");
    expect(bar.textContent).toContain("3 lines");
  });

  it("shows why recording stopped, and offers the resume that starts a new cell", () => {
    const onResume = vi.fn();
    render(
      <AnchorBar
        anchor={{
          ...anchored,
          suspended:
            "recording paused — this line looks like a secret\n   the shell ran it; the document did not record it",
        }}
        onResume={onResume}
        onUnanchor={() => {}}
      />,
    );
    const why = screen.getByTestId("anchor-bar-why");
    expect(why.textContent).toContain("looks like a secret");
    // The second half is the part that stops a person assuming the cell
    // holds what they did.
    expect(why.textContent).toContain("the shell ran it");
    // And it never overclaims.
    expect(why.textContent?.toLowerCase()).not.toContain("safe");

    fireEvent.click(screen.getByText("Resume"));
    expect(onResume).toHaveBeenCalled();
  });

  it("says a program is reading the keys, and offers no Resume for it", () => {
    // The distinction that matters: this one resolves itself when the
    // program exits, so offering a verb would invite a person to fix
    // something that is not broken.
    render(
      <AnchorBar
        anchor={{
          ...anchored,
          foreign:
            "not recording — python3 is reading these keys itself\n   the terminal is yours; the document resumes when it exits",
        }}
        onResume={() => {}}
        onUnanchor={() => {}}
      />,
    );
    const bar = screen.getByTestId("anchor-bar");
    expect(bar.textContent).toContain("waiting");
    expect(bar.textContent).toContain("python3");
    expect(bar.textContent).toContain("resumes when it exits");
    expect(screen.queryByText("Resume")).toBeNull();
  });

  it("shows the suspension when both are true, because only it has a verb", () => {
    render(
      <AnchorBar
        anchor={{
          ...anchored,
          suspended: "recording paused — this line looks like a secret\n   the shell ran it",
          foreign: "not recording — less is not a shell command\n   the terminal is yours",
        }}
        onResume={() => {}}
        onUnanchor={() => {}}
      />,
    );
    expect(screen.getByTestId("anchor-bar-why").textContent).toContain("looks like a secret");
    expect(screen.getByText("Resume")).toBeTruthy();
  });

  it("offers no Resume while nothing is suspended", () => {
    render(<AnchorBar anchor={anchored} onResume={() => {}} onUnanchor={() => {}} />);
    expect(screen.queryByText("Resume")).toBeNull();
    expect(screen.getByText("Stop writing")).toBeTruthy();
  });
});
</storycode:copy>

### Assemble `apps/web/src/terminal/AnchorBar.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/terminal/AnchorBar.test.tsx"><storycode:paste select="#f-635790b9cca6-expression_statement"/><storycode:paste select="#f-635790b9cca6-anchored"/><storycode:paste select="#f-635790b9cca6-expression_statement-2"/></storycode:file>

</storycode:doc>
