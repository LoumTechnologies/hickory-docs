// @vitest-environment jsdom
//
// Protects docs/guarantees/authoring/a-fence-becomes-a-cell-that-runs.md

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";

import { FenceConvert } from "./FenceConvert";

afterEach(cleanup);

function panel(props: Partial<React.ComponentProps<typeof FenceConvert>> = {}) {
  const onConvert = vi.fn();
  const onCancel = vi.fn();
  render(
    <FenceConvert
      info="python"
      body='print("hi")'
      containers={["py"]}
      onConvert={onConvert}
      onCancel={onCancel}
      {...props}
    />,
  );
  return { onConvert, onCancel };
}

describe("converting a fence", () => {
  it("shows the exec cell it would write before writing it", () => {
    panel();
    const preview = document.querySelector(".insert-menu__preview-text")!.textContent!;
    expect(preview).toContain('<hick:exec container="py">');
    expect(preview).toContain("python3 - <<'EOF'");
    expect(preview).toContain('print("hi")');
    expect(preview).toContain("</hick:exec>");
  });

  it("hands back exactly the previewed text", () => {
    const { onConvert } = panel();
    const preview = document.querySelector(".insert-menu__preview-text")!.textContent!;
    fireEvent.click(screen.getByRole("button", { name: "Convert" }));
    expect(onConvert).toHaveBeenCalledWith(preview);
  });

  it("offers the containers the document declares, and follows the choice", () => {
    panel({ containers: ["py", "node"] });
    fireEvent.change(screen.getByLabelText(/Container/), { target: { value: "node" } });
    expect(document.querySelector(".insert-menu__preview-text")!.textContent).toContain(
      '<hick:exec container="node">',
    );
  });

  it("asks for a container name when the document declares none", () => {
    const { onConvert } = panel({ containers: [] });
    const input = screen.getByLabelText(/Container/) as HTMLInputElement;
    expect(input.tagName).toBe("INPUT");
    // An exec with nowhere to run is not a conversion worth doing.
    expect((screen.getByRole("button", { name: "Convert" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    fireEvent.change(input, { target: { value: "py" } });
    fireEvent.click(screen.getByRole("button", { name: "Convert" }));
    expect(onConvert).toHaveBeenCalled();
  });

  it("says how a shell fence was treated", () => {
    panel({ info: "bash", body: "make build" });
    expect(screen.getByText(/already shell/)).toBeTruthy();
    expect(document.querySelector(".insert-menu__preview-text")!.textContent).toContain(
      "make build",
    );
  });

  it("warns rather than guesses for a language it has no interpreter for", () => {
    panel({ info: "sql", body: "SELECT 1;" });
    expect(screen.getByRole("note").textContent).toContain("sql");
  });

  it("cancels without converting", () => {
    const { onConvert, onCancel } = panel();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancel).toHaveBeenCalled();
    expect(onConvert).not.toHaveBeenCalled();
  });
});
