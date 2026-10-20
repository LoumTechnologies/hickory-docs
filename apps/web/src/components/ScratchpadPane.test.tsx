// @vitest-environment jsdom
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ScratchpadPane } from "./ScratchpadPane";

const saveScratchpad = vi.fn();
vi.mock("../api/client", () => ({
  api: {
    saveScratchpad: (text: string, assisted?: boolean) => assisted === undefined ? saveScratchpad(text) : saveScratchpad(text, assisted),
  },
}));

afterEach(() => {
  cleanup();
  saveScratchpad.mockReset();
});

describe("the scratchpad", () => {
  it("will not save nothing", async () => {
    render(<ScratchpadPane />);
    const button = screen.getByRole("button", {
      name: /save as note/i,
    }) as HTMLButtonElement;
    expect(button.disabled).toBe(true);
  });

  it("saves what was typed and says where it landed", async () => {
    saveScratchpad.mockResolvedValue({ path: "2026-08-18-standup.md" });
    render(<ScratchpadPane />);

    fireEvent.change(screen.getByLabelText(/scratchpad/i), {
      target: { value: "All green." },
    });
    fireEvent.click(screen.getByRole("button", { name: /save as note/i }));

    await waitFor(() =>
      expect(saveScratchpad).toHaveBeenCalledWith("All green."),
    );
    expect(await screen.findByText(/2026-08-18-standup\.md/)).toBeTruthy();
  });

  it("clears itself after a save, so the next thought starts empty", async () => {
    saveScratchpad.mockResolvedValue({ path: "note.md" });
    render(<ScratchpadPane />);

    const box = screen.getByLabelText(/scratchpad/i) as HTMLTextAreaElement;
    fireEvent.change(box, { target: { value: "Something" } });
    fireEvent.click(screen.getByRole("button", { name: /save as note/i }));

    await waitFor(() => expect(box.value).toBe(""));
  });

  it("shows the refusal instead of losing the text", async () => {
    // The refusals this can hit all carry a next step, and the words the
    // person typed are still in the box to act on.
    saveScratchpad.mockRejectedValue(
      new Error("this text contains `<hick:` — Next step: reword it"),
    );
    render(<ScratchpadPane />);

    const box = screen.getByLabelText(/scratchpad/i) as HTMLTextAreaElement;
    fireEvent.change(box, { target: { value: "I used a hick tag" } });
    fireEvent.click(screen.getByRole("button", { name: /save as note/i }));

    expect(await screen.findByText(/Next step/)).toBeTruthy();
    expect(box.value).toBe("I used a hick tag");
  });
});

it("retains agent-assisted authorship when a human saves an agent-edited draft", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  saveScratchpad.mockResolvedValue({ path: "assisted.md" });
  render(<UiActionContext.Provider value={registry}><ScratchpadPane /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, {
    handle: actions().find(action => action.label === "Set scratchpad draft")!.handle, arguments: { text: "Agent draft" }, request_id: "draft",
  }); });
  fireEvent.change(screen.getByLabelText(/scratchpad/i), { target: { value: "Agent draft with human edits" } });
  fireEvent.click(screen.getByRole("button", { name: /save as note/i }));
  await waitFor(() => expect(saveScratchpad).toHaveBeenCalledExactlyOnceWith("Agent draft with human edits", true));
  expect(await screen.findByText("Saved to assisted.md")).toBeTruthy();
});

it("excludes duplicate scratchpad saves and retains a failed agent draft", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let fail!: (error: unknown) => void;
  saveScratchpad.mockImplementation(() => new Promise((_done, reject) => { fail = reject; }));
  render(<UiActionContext.Provider value={registry}><ScratchpadPane /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  fireEvent.change(screen.getByLabelText(/scratchpad/i), { target: { value: "Retain me" } });
  const actor = { id: "acp:turn", kind: "agent" as const };
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, actor, {
    handle: actions().find(action => action.label === "Save scratchpad as note")!.handle, arguments: {}, request_id: "save",
  }); });
  expect(await registry.act(registry.scope, { id: "python:other", kind: "agent" }, {
    handle: actions().find(action => action.label === "Save scratchpad as note")!.handle, arguments: {}, request_id: "other",
  })).toMatchObject({ status: "disabled" });
  await act(async () => { fail(Error("Engine update required")); await pending; });
  expect((screen.getByLabelText(/scratchpad/i) as HTMLTextAreaElement).value).toBe("Retain me");
  expect(saveScratchpad).toHaveBeenCalledExactlyOnceWith("Retain me", true);
});
