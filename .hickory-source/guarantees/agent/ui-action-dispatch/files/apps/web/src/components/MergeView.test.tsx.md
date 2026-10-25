<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/MergeView.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `bad7329d39a0c7b889497875d82fe4824771dcb139837e76220b8d5c8007ff88`.

<book:entry path="apps/web/src/components/MergeView.test.tsx">
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { afterEach, describe, expect, it, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";

import { MergeView } from "./MergeView";

afterEach(() => {
  cleanup();
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
</book:entry>

</book:doc>
