<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/GithubBodyAndComment.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The Workspace Tree Is An Editor Buffer](../../../../../../guarantees/authoring/the-workspace-tree-edits-like-an-editor/contract.md)

Captured SHA-256: `7ce98a780b8cef1f1c3b13c02ff2c661e04ba2f8e9ac58f66c07a6ce2c8486ef`.

<book:entry path="apps/web/src/shell/GithubBodyAndComment.test.tsx">
// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { GithubBodyAndComment } from "./GithubBodyAndComment";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { installMockHandler } from "../api/client";
afterEach(cleanup);

it("shares provider drafts and exact replies while preserving a changed body draft", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const calls: unknown[] = [];
  installMockHandler(async (_method, _path, body) => { calls.push(body); return { ok: true }; });
  const changed = vi.fn();
  const mount = (body: string) => <UiActionContext.Provider value={registry}><ul><GithubBodyAndComment kind="issue" repository="acme/widget" number={7}
    body={body} comments={[]} onChanged={changed} /></ul></UiActionContext.Provider>;
  const view = render(mount("Original"));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope,
    { id: "python:turn", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  await run("Set GitHub body draft issue:acme/widget#7", { text: "Draft" });
  expect((screen.getByLabelText("Body") as HTMLTextAreaElement).value).toBe("Draft");
  view.rerender(mount("Provider changed"));
  expect(await run("Save GitHub body issue:acme/widget#7")).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  expect(calls).toEqual([]);
  expect((screen.getByLabelText("Body") as HTMLTextAreaElement).value).toBe("Draft");
  await run("Reset GitHub body draft issue:acme/widget#7");
  fireEvent.change(screen.getByLabelText("Body"), { target: { value: "Native draft" } });
  expect(await run("Save GitHub body issue:acme/widget#7")).toMatchObject({ status: "result", outcome: { value: { response: { ok: true }, independently_verified: false } } });
  await run("Set GitHub comment draft issue:acme/widget#7", { text: "Agent draft" });
  fireEvent.click(screen.getByRole("button", { name: "Add comment" }));
  await screen.findByText("Comment added.");
  expect(calls).toEqual([
    { kind: "issue", repository: "acme/widget", number: 7, field: "body", value: "Native draft" },
    { kind: "issue", repository: "acme/widget", number: 7, body: "Agent draft" },
  ]);
  expect(changed).toHaveBeenCalledTimes(2);
});

it("preserves a failed comment draft and excludes a second submission", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let fail!: (reason: unknown) => void;
  installMockHandler(async () => new Promise((_done, reject) => { fail = reject; }));
  render(<UiActionContext.Provider value={registry}><ul><GithubBodyAndComment kind="pr" repository="acme/widget" number={12}
    body="body" comments={[]} onChanged={() => {}} /></ul></UiActionContext.Provider>);
  fireEvent.change(screen.getByLabelText("New comment"), { target: { value: "Keep this" } });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:turn", kind: "agent" }, {
    handle: actions().find(action => action.label === "Add GitHub comment pr:acme/widget#12")!.handle, arguments: {}, request_id: "send",
  }); });
  expect(await registry.act(registry.scope, { id: "python:other", kind: "agent" }, {
    handle: actions().find(action => action.label === "Add GitHub comment pr:acme/widget#12")!.handle, arguments: {}, request_id: "other",
  })).toMatchObject({ status: "disabled" });
  let result: unknown;
  await act(async () => { fail(Error("Provider refused")); result = await pending; });
  expect(result).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("may have applied effects") } });
  expect((screen.getByLabelText("New comment") as HTMLTextAreaElement).value).toBe("Keep this");
});
</book:entry>

</book:doc>
