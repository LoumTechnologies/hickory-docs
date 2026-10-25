<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/FenceConvert.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Fence Becomes A Cell That Runs](../../../../../contract.md)

Captured SHA-256: `d9acb6449a0d26d7780b0dd86b46cad7e039ede0418b3769149439b5e715df5c`.

<book:entry path="apps/web/src/components/FenceConvert.test.tsx">
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

it("shares the conversion draft and preview with agents and retires replaced-source handles", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const convert = vi.fn(() => true), cancel = vi.fn();
  const panel = (body: string) => <UiActionContext.Provider value={registry}><FenceConvert info="python" body={body} containers={["py", "other"]} onConvert={convert} onCancel={cancel} /></UiActionContext.Provider>;
  const view = render(panel("print(1)"));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  const read = await run("Read fence conversion preview bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(Uint8Array.from(read.outcome.value.bytes))).toContain("print(1)");
  await run("Set fence conversion container", { container: "other" });
  expect((screen.getByLabelText(/Container/) as HTMLSelectElement).value).toBe("other");
  const old = actions().find(action => action.label === "Convert fence to executable cell")!.handle;
  view.rerender(panel("print(2)"));
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: old, arguments: {}, request_id: "stale" })).not.toMatchObject({ status: "result" });
  expect(convert).not.toHaveBeenCalled();
  const preview = document.querySelector(".insert-menu__preview-text")!.textContent!;
  expect(await run("Convert fence to executable cell")).toMatchObject({ status: "result", outcome: { value: { source_changed: true, saved: false, executed: false } } });
  expect(convert).toHaveBeenCalledExactlyOnceWith(preview);
  await run("Cancel fence conversion");
  expect(cancel).toHaveBeenCalledTimes(1);
});
it("preserves the conversion draft when the owner refuses changed source", () => {
  panel({ onConvert: () => false });
  fireEvent.click(screen.getByRole("button", { name: "Convert" }));
  expect(screen.getByRole("alert").textContent).toContain("source or editor authority changed");
  expect((screen.getByLabelText(/Container/) as HTMLSelectElement).value).toBe("py");
});
</book:entry>

</book:doc>
