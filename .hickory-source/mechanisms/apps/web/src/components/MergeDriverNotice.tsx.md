<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/MergeDriverNotice.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c97d597c05fd"></a>
## `apps/web/src/components/MergeDriverNotice.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Missing Merge Driver Is A Button, Not A Command To Go And Type](../../../../../guarantees/collaboration/a-missing-merge-driver-is-a-button/contract.md), [`.md` Documents Merge Through Hick, And A Clone That Cannot Is Told](../../../../../guarantees/collaboration/hick-documents-merge-through-hick/contract.md).

<a id="f-c97d597c05fd-MergeDriverNotice"></a>
### `MergeDriverNotice`

<storycode:copy id="f-c97d597c05fd-MergeDriverNotice">
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
// Whether `.md` documents merge through hick — asked once, at project open.
//
// The sharp edge this exists for: `.gitattributes` routing (`*.md
// merge=hick`) is committed and reaches every clone, but the driver
// DEFINITION cannot be — git will not let a repository hand a clone an
// executable command. A clone that never ran `hick init` therefore merges
// `.md` files with git's line merge, **silently**: no warning, no marker,
// nothing to notice afterwards.
//
// So the check cannot live in the pre-commit hook, because `hick init`
// installs the hook: the clone that is missing the driver is exactly the
// clone that is missing the thing that would report it. It belongs somewhere
// that runs without having been installed — here, and in `hick test`. And it
// belongs at OPEN rather than at commit, because a check at commit time tells
// you after the damage.
//
// See docs/specs/freeform/provenance-across-versions.md.

import { useEffect, useId, useRef, useState } from "react";

import { api } from "../api/client";
import type { MergeDriverStatus } from "../api/types";

/** Dismissal is per window, not persisted: this is a fact about the clone,
 * and a person who dismissed it last month in another checkout has not fixed
 * anything here. */
export function MergeDriverNotice() {
  const [status, setStatus] = useState<MergeDriverStatus | null>(null);
  const [dismissed, setDismissed] = useState(false);
  // The button's own state: running, what it did, or why it could not.
  const inFlight = useRef(false);
  const [running, setRunning] = useState(false);
  const [done, setDone] = useState<string | null>(null);
  const [failed, setFailed] = useState<string | null>(null);

  // A fixable failure is a button, never a command to go and type
  // (docs/guarantees/collaboration/a-missing-merge-driver-is-a-button.md).
  // This runs the same `hick init` the sentence names, in the engine.
  const runInit = async (): Promise<UiOutcome> => {
    if (inFlight.current) return { status: "unavailable", reason: "Repository initialization is already pending" };
    inFlight.current = true; setRunning(true); setFailed(null);
    try {
      const outcome = await api.initRepository();
      const wrote = Object.entries(outcome.changed).filter(([, changed]) => changed).map(([what]) => what.replace("_", "."));
      setDone(outcome.ok
        ? `hick init ran: ${wrote.length ? wrote.join(", ") : "everything was already in place"}. \`.md\` documents now merge through hick in this clone.`
        : `hick init ran, but the driver is still not defined: ${outcome.status.summary}`);
      return outcome.ok ? { status: "completed", value: { reply: JSON.stringify(outcome), scope: "Actual initialization reply; not a merge execution witness" } } : { status: "unavailable", reason: outcome.status.summary };
    } catch (e) {
      const reason = e instanceof Error ? e.message : String(e); setFailed(reason); return { status: "unavailable", reason };
    } finally { inFlight.current = false; setRunning(false); }
  };

  useEffect(() => {
    let live = true;
    api.mergeDriver().then(
      (answer) => {
        // Only when something is wrong. A banner confirming that things work
        // is a banner people learn to scroll past.
        if (live && answer.status.repository && !answer.ok) setStatus(answer.status);
      },
      // A folder that is not a repository, or a machine with no git, is not a
      // problem to report — it is a folder of notes.
      () => {},
    );
    return () => {
      live = false;
    };
  }, []);

  const parent = `merge-driver:${useId()}`;
  const revision = useUiRevision(() => [status, dismissed, running, done, failed]);
  const commands: { key: string; command: UiCommand }[] = [];
  if (status && !dismissed) {
    if (!done) commands.push({ key: `${parent}:init`, command: { parent, subject: parent, label: "Initialize repository merge driver", parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: !running }), execute: runInit } });
    commands.push({ key: `${parent}:dismiss`, command: { parent, subject: `${parent}:visibility`, label: "Dismiss merge driver notice", parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: true }), execute: async () => { setDismissed(true); return { status: "completed", value: { dismissed: true, repaired: false } }; } } });
  }
  const invoke = useUiCommands(commands);
  const human = (key: string) => void invoke(`${parent}:${key}`).catch(() => undefined);
  const evidence = <MergeDriverReading value={{ status, running, done, failed, scope: "Host merge-driver configuration and current repair state; not proof of an executed merge" }} />;
  if (!status || dismissed) return null;
  if (done) {
    return (
      <div className="banner" role="status">
        {evidence}
        <span>{done}</span>
        <button type="button" className="banner__dismiss" onClick={() => human("dismiss")}>
          Dismiss
        </button>
      </div>
    );
  }
  return (
    <div className="banner banner-warn" role="status">
      {evidence}
      <span>{failed ? `${status.summary} — hick init failed: ${failed}` : status.summary}</span>
      <button type="button" className="banner__action" onClick={() => human("init")} disabled={running}>
        {running ? "Running hick init…" : "Run hick init"}
      </button>
      <button
        type="button"
        className="banner__dismiss"
        onClick={() => human("dismiss")}
      >
        Dismiss
      </button>
    </div>
  );
}

</storycode:copy>
<a id="f-c97d597c05fd-MergeDriverReading"></a>
### `MergeDriverReading`

<storycode:copy id="f-c97d597c05fd-MergeDriverReading">
function MergeDriverReading({ value }: { value: unknown }) { useUiReading("merge driver notice", value); return null; }
</storycode:copy>

### Assemble `apps/web/src/components/MergeDriverNotice.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/MergeDriverNotice.tsx"><storycode:paste select="#f-c97d597c05fd-MergeDriverNotice"/><storycode:paste select="#f-c97d597c05fd-MergeDriverReading"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-38c5ddfe6997"></a>
## `apps/web/src/components/MergeDriverNotice.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [A Missing Merge Driver Is A Button, Not A Command To Go And Type](../../../../../guarantees/collaboration/a-missing-merge-driver-is-a-button/contract.md), [`.md` Documents Merge Through Hick, And A Clone That Cannot Is Told](../../../../../guarantees/collaboration/hick-documents-merge-through-hick/contract.md).

<a id="f-38c5ddfe6997-answer"></a>
### `answer`

<storycode:copy id="f-38c5ddfe6997-answer">
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

</storycode:copy>
<a id="f-38c5ddfe6997-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-38c5ddfe6997-expression_statement">
beforeEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

</storycode:copy>
<a id="f-38c5ddfe6997-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-38c5ddfe6997-expression_statement-2">
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

</storycode:copy>
<a id="f-38c5ddfe6997-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-38c5ddfe6997-expression_statement-3">
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

</storycode:copy>
<a id="f-38c5ddfe6997-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-38c5ddfe6997-expression_statement-4">
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

</storycode:copy>
<a id="f-38c5ddfe6997-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-38c5ddfe6997-expression_statement-5">
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
</storycode:copy>

### Assemble `apps/web/src/components/MergeDriverNotice.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/MergeDriverNotice.test.tsx"><storycode:paste select="#f-38c5ddfe6997-answer"/><storycode:paste select="#f-38c5ddfe6997-expression_statement"/><storycode:paste select="#f-38c5ddfe6997-expression_statement-2"/><storycode:paste select="#f-38c5ddfe6997-expression_statement-3"/><storycode:paste select="#f-38c5ddfe6997-expression_statement-4"/><storycode:paste select="#f-38c5ddfe6997-expression_statement-5"/></storycode:file>


</storycode:doc>
