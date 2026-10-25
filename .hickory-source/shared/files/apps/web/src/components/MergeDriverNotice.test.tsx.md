<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/MergeDriverNotice.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [A Missing Merge Driver Is A Button, Not A Command To Go And Type](../../../../../../guarantees/collaboration/a-missing-merge-driver-is-a-button/contract.md)
- [`.md` Documents Merge Through Hick, And A Clone That Cannot Is Told](../../../../../../guarantees/collaboration/hick-documents-merge-through-hick/contract.md)

Captured SHA-256: `fedb039aac510654d3a5ef07ff011cb3b72a0fa9f421dec58e5972ded9ca2435`.

<book:entry path="apps/web/src/components/MergeDriverNotice.test.tsx">
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
// docs/guarantees/collaboration/hick-documents-merge-through-hick.md
import { describe, expect, it, vi, beforeEach } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { api } from "../api/client";
import { MergeDriverNotice } from "./MergeDriverNotice";

const answer = (over: Partial<{ repository: boolean; attributes: boolean; configured: boolean }>) =>
  vi.spyOn(api, "mergeDriver").mockResolvedValue({
    status: {
      repository: true,
      attributes: true,
      configured: false,
      summary:
        "`.gitattributes` routes `*.md` at the `hick` merge driver, but this clone has not defined it — so git is SILENTLY falling back to its line merge. Next step: run `hick init` in this repository.",
      ...over,
    },
    ok: false,
  });

beforeEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("a clone whose merge driver is not defined", () => {
  it("is told at open, because a check at commit time tells you after the damage", async () => {
    answer({});
    render(<MergeDriverNotice />);
    await waitFor(() => expect(screen.getByText(/SILENTLY/)).toBeTruthy());
    expect(screen.getByRole("button", { name: "Run hick init" })).toBeTruthy();
  });

  it("can be dismissed for this window", async () => {
    answer({});
    render(<MergeDriverNotice />);
    await waitFor(() => expect(screen.getByText(/SILENTLY/)).toBeTruthy());
    act(() => {
      fireEvent.click(screen.getByRole("button", { name: /dismiss/i }));
    });
    await waitFor(() => expect(screen.queryByText(/SILENTLY/)).toBeNull());
  });
});

describe("a clone that is wired up", () => {
  it("says nothing — a banner confirming things work is one people learn to skip", async () => {
    vi.spyOn(api, "mergeDriver").mockResolvedValue({
      status: { repository: true, attributes: true, configured: true, summary: "fine" },
      ok: true,
    });
    const { container } = render(<MergeDriverNotice />);
    await waitFor(() => expect(container.textContent).toBe(""));
  });

  it("says nothing for a folder that is not a repository", async () => {
    vi.spyOn(api, "mergeDriver").mockRejectedValue(new Error("not a repository"));
    const { container } = render(<MergeDriverNotice />);
    await waitFor(() => expect(container.textContent).toBe(""));
  });
});

// docs/guarantees/collaboration/a-missing-merge-driver-is-a-button.md
describe("the banner's button", () => {
  it("runs hick init in the engine and says what it wrote", async () => {
    answer({});
    const init = vi.spyOn(api, "initRepository").mockResolvedValue({
      changed: {
        hook: true,
        gitignore: false,
        gitattributes: false,
        merge_driver: true,
        agents_md: false,
        mcp_json: false,
      },
      hook_path: ".git/hooks/pre-commit",
      status: { repository: true, attributes: true, configured: true, summary: "ok" },
      ok: true,
    });
    render(<MergeDriverNotice />);
    await waitFor(() => expect(screen.getByText(/SILENTLY/)).toBeTruthy());
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Run hick init" }));
    });
    expect(init).toHaveBeenCalledTimes(1);
    await waitFor(() => expect(screen.getByText(/hick init ran: hook, merge.driver/)).toBeTruthy());
    expect(screen.queryByText(/SILENTLY/)).toBeNull();
  });

  it("keeps the warning and says why when hick init fails", async () => {
    answer({});
    vi.spyOn(api, "initRepository").mockRejectedValue(new Error("not a git work tree"));
    render(<MergeDriverNotice />);
    await waitFor(() => expect(screen.getByText(/SILENTLY/)).toBeTruthy());
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Run hick init" }));
    });
    await waitFor(() => expect(screen.getByText(/hick init failed: not a git work tree/)).toBeTruthy());
    expect(screen.getByRole("button", { name: "Run hick init" })).toBeTruthy();
  });
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares the awaited repair with agents and retires controls after dismissal", async () => {
  answer({});
  const outcome = { ok: true, changed: { hook: true, gitignore: false, gitattributes: false, merge_driver: true, agents_md: false, mcp_json: false }, hook_path: ".git/hooks/pre-commit", status: { repository: true, attributes: true, configured: true, summary: "configured" } };
  let finish!: (value: typeof outcome) => void;
  const init = vi.spyOn(api, "initRepository").mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><MergeDriverNotice /></UiActionContext.Provider>);
  await screen.findByRole("button", { name: "Run hick init" });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  const repair = actions().find(a => a.label === "Initialize repository merge driver")!;
  let result!: ReturnType<typeof registry.act>;
  await act(async () => { result = registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: repair.handle, arguments: {}, request_id: "init" }); });
  expect(init).toHaveBeenCalledOnce();
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: repair.handle, arguments: {}, request_id: "duplicate" })).toMatchObject({ status: "stale" });
  let response: unknown;
  await act(async () => { finish(outcome); response = await result; });
  expect(response).toMatchObject({ status: "result", outcome: { status: "completed", value: { reply: JSON.stringify(outcome) } } });
  expect(actions().some(a => a.label === "Initialize repository merge driver")).toBe(false);
  const dismiss = actions().find(a => a.label === "Dismiss merge driver notice")!;
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: dismiss.handle, arguments: {}, request_id: "dismiss" }); });
  expect(actions()).toHaveLength(0);
});
</book:entry>

</book:doc>
