// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AnchorBar } from "./AnchorBar";

afterEach(cleanup);

const anchored = {
  doc: "abc123",
  container: "sdk",
  suspended: null,
  foreign: null,
  recorded: 3,
};

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
        docName="scaffolding.hick"
        onResume={() => {}}
        onUnanchor={() => {}}
      />,
    );
    const bar = screen.getByTestId("anchor-bar");
    expect(bar.textContent).toContain("writing");
    expect(bar.textContent).toContain("scaffolding.hick");
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
