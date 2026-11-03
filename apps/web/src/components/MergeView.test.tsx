import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";

import { DiskChangeNotice, MergeView } from "./MergeView";

afterEach(() => {
  cleanup();
  localStorage.clear();
});

const view = (props: Partial<React.ComponentProps<typeof MergeView>> = {}) =>
  render(
    <MergeView
      path="notes.md"
      base={"one\ntwo\nthree\n"}
      ours={"one\nOURS\nthree\n"}
      theirs={"one\nTHEIRS\nthree\n"}
      onAccept={() => {}}
      onCancel={() => {}}
      {...props}
    />,
  );

describe("the merge view says how much of this is your problem", () => {
  it("leads with the count that has to be answered", () => {
    // Before reading a word of the merge, the reader needs to know whether
    // this is two decisions or forty.
    view();
    expect(screen.getByRole("status").textContent).toMatch(/1 of 1 still needs you/i);
  });

  it("says so plainly when there is nothing to decide", () => {
    view({ ours: "one\nOURS\nthree\n", theirs: "one\ntwo\nthree\n" });
    expect(screen.getByRole("status").textContent).toMatch(/merged cleanly/i);
    expect(screen.queryAllByRole("group", { name: /conflict/i })).toHaveLength(0);
  });

  it("counts down as answers are given", () => {
    view();
    fireEvent.click(screen.getByRole("button", { name: "Keep theirs" }));
    expect(screen.getByRole("status").textContent).toMatch(/0 of 1 still need you/i);
  });
});

describe("the merge view never takes a side silently", () => {
  it("shows an auto-merged region and names who it came from", () => {
    // A merge tool that silently took a side is a merge tool nobody trusts
    // twice.
    view({ ours: "ONE\ntwo\nthree\n", theirs: "one\ntwo\nTHREE\n" });
    const labels = screen.getAllByText(/^Merged —/).map((el) => el.textContent);
    expect(labels.join(" ")).toMatch(/only your unsaved changes touched this/i);
    expect(labels.join(" ")).toMatch(/only the file on disk changed this/i);
  });

  it("shows both sides of a conflict, labelled", () => {
    view();
    const conflict = screen.getByRole("group", { name: /conflict 1/i });
    expect(within(conflict).getByText("Your unsaved changes")).toBeTruthy();
    expect(within(conflict).getByText("The file on disk")).toBeTruthy();
  });
});

describe("answering a conflict", () => {
  it("produces the chosen text on accept", () => {
    const onAccept = vi.fn();
    view({ onAccept });
    fireEvent.click(screen.getByRole("button", { name: "Keep theirs" }));
    fireEvent.click(screen.getByRole("button", { name: /accept/i }));
    expect(onAccept).toHaveBeenCalledWith("one\nTHEIRS\nthree\n");
  });

  it("keeps our side for anything left undecided, and says so on the button", () => {
    // Accepting with an unanswered conflict is a real choice, not a mistake to
    // block: it means "everything I did not answer stays mine".
    const onAccept = vi.fn();
    view({ onAccept });
    const accept = screen.getByRole("button", { name: /accept, keeping mine where undecided/i });
    fireEvent.click(accept);
    expect(onAccept).toHaveBeenCalledWith("one\nOURS\nthree\n");
  });

  it("lets an answer be taken back by pressing it again", () => {
    view();
    const theirs = screen.getByRole("button", { name: "Keep theirs" });
    fireEvent.click(theirs);
    expect(theirs.getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(theirs);
    expect(theirs.getAttribute("aria-pressed")).toBe("false");
  });

  it("cancels without touching anything", () => {
    const onCancel = vi.fn();
    const onAccept = vi.fn();
    view({ onCancel, onAccept });
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancel).toHaveBeenCalled();
    expect(onAccept).not.toHaveBeenCalled();
  });
});

describe("with no common ancestor", () => {
  it("says why every difference is a question", () => {
    view({ base: "" });
    expect(screen.getByText(/no common ancestor/i)).toBeTruthy();
  });

  it("offers no 'keep neither', because there is no before to go back to", () => {
    view({ base: "" });
    expect(screen.queryByRole("button", { name: "Keep neither" })).toBeNull();
  });
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares merge preview choices and refuses a live changed owner without fabricating approval", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let current = true;
  const accept = vi.fn(async () => true);
  render(<UiActionContext.Provider value={registry}><MergeView path="x" base="base" ours="PRIVATE OURS" theirs="theirs" sourceCurrent={() => current} onAccept={accept} onCancel={vi.fn()} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(g => registry.observe(g).actions);
  expect(JSON.stringify(actions())).not.toContain("PRIVATE OURS");
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  await run("Choose merge conflict resolution", { index: 0, choice: "theirs" });
  const old = actions().find(a => a.label === "Accept merge preview")!;
  current = false;
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "changed" })).toMatchObject({ status: "stale" });
  expect(accept).not.toHaveBeenCalled();
  current = true;
  expect(await run("Accept merge preview")).toMatchObject({ status: "result", outcome: { status: "completed", value: { approved: false, verified: false } } });
  expect(accept).toHaveBeenCalledWith("theirs");
});

it("retires old answers when the merge inputs are replaced", () => {
  const accept = vi.fn();
  const mounted = view({ onAccept: accept });
  fireEvent.click(screen.getByRole("button", { name: "Keep theirs" }));
  mounted.rerender(<MergeView path="notes.md" base="base" ours="new ours" theirs="new theirs" onAccept={accept} onCancel={vi.fn()} />);
  fireEvent.click(screen.getByRole("button", { name: /accept, keeping mine/i }));
  expect(accept).toHaveBeenCalledWith("new ours");
});
describe("disk change choices", () => {
  const mount = (source = "base", apply = vi.fn(), read = vi.fn().mockResolvedValue({ source: "disk" })) => {
    const props = { path: "code.md", base: "base", source, read, apply };
    return { ...render(<DiskChangeNotice {...props} />), props, read, apply };
  };
  it("announces a clean change without applying it until reload", async () => {
    const { apply } = mount();
    fireEvent.click(await screen.findByRole("button", { name: "Reload from disk" }));
    await act(async () => {});
    expect(apply).toHaveBeenCalledWith({ source: "disk" }, "disk");
  });
  it("ignores a disk version without applying or saving anything", async () => {
    const { apply } = mount("unsaved");
    fireEvent.click(await screen.findByRole("button", { name: "Ignore disk changes" }));
    fireEvent(window, new Event("focus"));
    await act(async () => {});
    expect(screen.queryByRole("button", { name: "Merge changes" })).toBeNull();
    expect(apply).not.toHaveBeenCalled();
  });
  it("persists safe reload preference and automatically reloads a clean buffer", async () => {
    const { apply } = mount();
    fireEvent.click(await screen.findByRole("checkbox"));
    await act(async () => {});
    expect(localStorage.getItem("hickory.reload-unmodified")).toBe("true");
    expect(apply).toHaveBeenCalledWith({ source: "disk" }, "disk");
  });
  it("never auto reloads an unsaved buffer", async () => {
    localStorage.setItem("hickory.reload-unmodified", "true");
    const { apply } = mount("unsaved");
    expect(await screen.findByRole("button", { name: "Reload and discard unsaved changes" })).toBeTruthy();
    expect(screen.queryByRole("checkbox")).toBeNull();
    expect(apply).not.toHaveBeenCalled();
  });
  it("offers the existing three-way merge and preserves undecided local text", async () => {
    const { apply } = mount("unsaved");
    fireEvent.click(await screen.findByRole("button", { name: "Merge changes" }));
    fireEvent.click(screen.getByRole("button", { name: "Accept, keeping mine where undecided" }));
    await act(async () => {});
    expect(apply).toHaveBeenCalledWith({ source: "disk" }, "unsaved");
  });
  it("accepts free edits made in the literate merge result", async () => {
    const { apply } = mount("unsaved");
    fireEvent.click(await screen.findByRole("button", { name: "Merge changes" }));
    const preview = await screen.findByRole("region", { name: "Literate merge result" }, { timeout: 10000 });
    const { EditorView } = await import("@codemirror/view");
    const editor = EditorView.findFromDOM(preview.querySelector<HTMLElement>(".cm-editor")!)!;
    await act(async () => {});
    act(() => { editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "reviewed result" }, userEvent: "input" }); });
    fireEvent.click(screen.getByRole("button", { name: "Accept, keeping mine where undecided" }));
    await act(async () => {});
    expect(apply).toHaveBeenCalledWith({ source: "disk" }, "reviewed result");
  }, 15000);
  it("refuses a reload when disk changes again", async () => {
    const { read, apply } = mount("unsaved");
    await screen.findByRole("button", { name: "Reload and discard unsaved changes" });
    read.mockResolvedValue({ source: "new disk" });
    fireEvent.click(screen.getByRole("button", { name: "Reload and discard unsaved changes" }));
    expect(await screen.findByRole("alert")).toHaveProperty("textContent", "The file changed again. Review its latest version.");
    expect(apply).not.toHaveBeenCalled();
  });
  it("keeps typing that arrives while reload is reading disk", async () => {
    const { read, apply, rerender, props } = mount();
    await screen.findByRole("button", { name: "Reload from disk" });
    let finish!: (disk: { source: string }) => void;
    read.mockImplementation(() => new Promise(resolve => { finish = resolve; }));
    fireEvent.click(screen.getByRole("button", { name: "Reload from disk" }));
    rerender(<DiskChangeNotice {...props} source="new typing" />);
    await act(async () => finish({ source: "disk" }));
    expect(apply).not.toHaveBeenCalled();
    expect(screen.getByRole("alert").textContent).toContain("newer edits were kept");
  });
  it("does not apply an in-flight reload after closing its view", async () => {
    const { read, apply, unmount } = mount();
    await screen.findByRole("button", { name: "Reload from disk" });
    let finish!: (disk: { source: string }) => void;
    read.mockImplementation(() => new Promise(resolve => { finish = resolve; }));
    fireEvent.click(screen.getByRole("button", { name: "Reload from disk" }));
    unmount();
    await act(async () => finish({ source: "disk" }));
    expect(apply).not.toHaveBeenCalled();
  });
});
