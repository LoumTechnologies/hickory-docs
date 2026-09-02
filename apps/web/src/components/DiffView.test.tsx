// Protects docs/guarantees/collaboration/the-git-pane-does-the-daily-loop.md
import { afterEach, describe, expect, it } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { DiffView, diffLineKind } from "./DiffView";

afterEach(cleanup);

describe("a diff's lines", () => {
  it("are told apart by their first character", () => {
    expect(diffLineKind("@@ -1,2 +1,3 @@")).toBe("hunk");
    expect(diffLineKind("+added")).toBe("add");
    expect(diffLineKind("-gone")).toBe("del");
    expect(diffLineKind(" same")).toBe("ctx");
    expect(diffLineKind("diff --git a/x b/x")).toBe("meta");
    expect(diffLineKind("+++ b/x")).toBe("meta");
    expect(diffLineKind("--- a/x")).toBe("meta");
  });

  it("render with a class per kind, so a reader sees what changed", () => {
    render(
      <DiffView
        path="a.txt"
        staged={false}
        binary={false}
        diff={"diff --git a/a.txt b/a.txt\n@@ -1 +1,2 @@\n one\n+two\n"}
      />,
    );
    expect(document.querySelectorAll(".diff-line--add")).toHaveLength(1);
    expect(document.querySelectorAll(".diff-line--hunk")).toHaveLength(1);
    expect(screen.getByText("unstaged")).toBeTruthy();
  });

  it("says a binary file has no lines rather than showing an empty diff", () => {
    render(<DiffView path="logo.png" staged={false} binary diff="Binary files differ\n" />);
    expect(screen.getByText(/binary file/i)).toBeTruthy();
  });
});
