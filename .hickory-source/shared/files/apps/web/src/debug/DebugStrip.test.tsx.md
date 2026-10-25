<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/DebugStrip.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [A Compiled Language Launches What A Build Produced](../../../../../../guarantees/debugging/a-compiled-language-launches-what-a-build-produced/contract.md)
- [Debug installs a missing supported adapter and continues](../../../../../../guarantees/debugging/a-missing-debugger-is-a-button/contract.md)
- [A Plain File Has The Same Debugger A Document Has](../../../../../../guarantees/debugging/a-plain-file-has-the-same-debugger/contract.md)
- [An Exception Breakpoint Uses The Adapter's Own Filters](../../../../../../guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters/contract.md)
- [Debug controls preserve editor space](../../../../../../guarantees/debugging/debug-controls-preserve-editor-space/contract.md)
- [Every Shortcut Is A Setting](../../../../../../guarantees/editor-intelligence/every-shortcut-is-a-setting/contract.md)

Captured SHA-256: `80ebeece59b82bf0741193b234b10fa0ee8dc8f0d026443efd9706254574d118`.

<book:entry path="apps/web/src/debug/DebugStrip.test.tsx">
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { isAction } from "../lib/keymap";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { DebugStrip, debugKeysActive, frameLabel, stepForKey } from "./DebugStrip";
import type { DebugStripProps } from "./DebugStrip";
import type { TranscriptEvent } from "../api/types";

describe("the combo box's line for a frame", () => {
  it("names the function and the line it will return to", () => {
    expect(
      frameLabel({ id: 2, name: "line_total", line: 14, source: "orders.py", in_document: true }),
    ).toBe("line_total — line 15");
  });

  it("says external for a frame outside the document", () => {
    // A library frame has no line HERE, and inventing one would point at
    // whatever prose happens to share the number.
    expect(
      frameLabel({ id: 9, name: "json.loads", line: null, source: "json/__init__.py", in_document: false }),
    ).toBe("json.loads — external");
    expect(
      frameLabel({ id: 3, name: "runner", line: null, source: "orders.py", in_document: true }),
    ).toBe("runner — external");
  });

  it("names a frame in another file of the folder by that file and line", () => {
    // Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
    // The callee in `tools/helpers.py` is not external: the app can open it,
    // and the stack should say where it is rather than shrug.
    expect(
      frameLabel({ id: 4, name: "double", line: null, source: "tools/helpers.py", source_line: 1, in_document: false }),
    ).toBe("double — tools/helpers.py:2");
    expect(
      frameLabel({ id: 5, name: "loads", line: null, source: "/usr/lib/python3/json/__init__.py", source_line: 3, in_document: false }),
    ).toBe("loads — external");
  });
});

describe("the function keys the tooltips promise", () => {
  it("maps F5, F10 and F11 to the steps their tooltips name", () => {
    // Continue is the keymap's (`debug.continue`), not this table's.
    expect(stepForKey("F5", false)).toBeNull();
    expect(isAction({ key: "F5", ctrlKey: false, metaKey: false, altKey: false, shiftKey: false }, "debug.continue")).toBe(true);
    expect(stepForKey("F10", false)).toBe("over");
    expect(stepForKey("F11", false)).toBe("in");
    expect(stepForKey("F11", true)).toBe("out");
  });

  it("claims no other key", () => {
    // Shift-F5 and Shift-F10 belong to the browser (reload, context menu);
    // taking them would be taking keys the tooltips never promised.
    expect(stepForKey("F5", true)).toBeNull();
    expect(stepForKey("F10", true)).toBeNull();
    expect(stepForKey("F12", false)).toBeNull();
    expect(stepForKey("Enter", false)).toBeNull();
  });

  it("acts only while paused", () => {
    // The guard that keeps F5 as the browser's reload the rest of the time.
    expect(debugKeysActive("paused")).toBe(true);
    for (const status of ["idle", "starting", "running", "finished", "failed"] as const) {
      expect(debugKeysActive(status)).toBe(false);
    }
  });
});

// Shared by every strip test, including the ones below this file's first
// describe — it was scoped inside that one and the next block could not see it.
const base = (over: Partial<DebugStripProps> = {}): DebugStripProps => ({
  status: "paused",
  program: "orders.py",
  message: null,
  capabilities: null,
  exceptionFilters: [],
  onToggleExceptionFilter: () => {},
  frames: [
    { id: 2, name: "line_total", line: 14, source: "orders.py", in_document: true },
    { id: 3, name: "<module>", line: 20, source: "orders.py", in_document: true },
  ],
  selectedFrame: 2,
  watches: [],
  onSelectFrame: () => {},
  onStep: () => {},
  onJumpHere: () => {},
  onStart: () => {},
  onStop: () => {},
  onAddWatch: () => {},
  onRemoveWatch: () => {},
  ...over,
});

describe("the strip on the debugged block", () => {
  // Without cleanup, a strip from an earlier test keeps its key listener and
  // preventDefaults the F10 the next test fires.
  afterEach(cleanup);


  it("shows the build in a terminal while starting, and keeps it when the start failed", () => {
    // The spinner is the wrong answer: a build that fails says why in its
    // own output, and a `Building…` label throws that away.
    const built: TranscriptEvent[] = [
      { t: 0, kind: "cmd", data: "dotnet build --nologo --configuration Debug (app/app.csproj)" },
      { t: 400, kind: "err", data: "Program.cs(1,25): error CS1519: Invalid token\n" },
      { t: 500, kind: "exit", code: 1 },
    ];
    const { rerender } = render(
      <DebugStrip {...base({ status: "starting", buildOutput: built })} />,
    );
    expect(screen.getByTestId("debug-build")).toBeTruthy();
    expect(screen.getByTestId("watch-terminal")).toBeTruthy();

    // The failing case is the one it exists for.
    rerender(
      <DebugStrip {...base({ status: "failed", buildOutput: built, message: "build failed" })} />,
    );
    expect(screen.getByTestId("debug-build")).toBeTruthy();

    // Once the program is running, a successful build's log is a wall of
    // text between a person and the frame they are looking at.
    rerender(<DebugStrip {...base({ status: "paused", buildOutput: built })} />);
    expect(screen.queryByTestId("debug-build")).toBeNull();
  });

  it("says the build is not evidence, where the build is shown", () => {
    // The transcript is the record; the terminal is the run happening.
    // Nothing is ever verified against what a terminal showed, and the one
    // place that could be misread is beside a debugger.
    render(
      <DebugStrip
        {...base({ status: "starting", buildOutput: [{ t: 0, kind: "out", data: "ok\n" }] })}
      />,
    );
    expect(screen.getByTestId("debug-build").textContent).toContain("not a transcript");
  });

  it("shows no build chrome for a language whose generated file IS the program", () => {
    render(<DebugStrip {...base({ status: "starting", buildOutput: [] })} />);
    expect(screen.queryByTestId("debug-build")).toBeNull();
  });

  it("is nothing while idle: the Debug chip is the way in", () => {
    const { container } = render(<DebugStrip {...base({ status: "idle" })} />);
    expect(container.firstChild).toBeNull();
  });

  it("offers the stack as a combo box that selects a frame", () => {
    const onSelectFrame = vi.fn();
    render(<DebugStrip {...base({ onSelectFrame })} />);
    const select = screen.getByLabelText("Call stack") as HTMLSelectElement;
    expect([...select.options].map((option) => option.text)).toEqual([
      "line_total — line 15",
      "<module> — line 21",
    ]);
    fireEvent.change(select, { target: { value: "3" } });
    expect(onSelectFrame).toHaveBeenCalledWith(3);
  });

  it("steps on F10 while paused, and never while running", () => {
    const onStep = vi.fn();
    const { rerender } = render(<DebugStrip {...base({ onStep })} />);
    fireEvent.keyDown(window, { key: "F10" });
    expect(onStep).toHaveBeenCalledWith("over");

    onStep.mockClear();
    rerender(<DebugStrip {...base({ onStep, status: "running" })} />);
    fireEvent.keyDown(window, { key: "F10" });
    expect(onStep).not.toHaveBeenCalled();
  });

  it("leaves a modified chord to the browser", () => {
    // Ctrl-F5 is a hard reload; a debugger that eats it is a debugger that
    // broke the browser.
    const onStep = vi.fn();
    render(<DebugStrip {...base({ onStep })} />);
    fireEvent.keyDown(window, { key: "F5", ctrlKey: true });
    expect(onStep).not.toHaveBeenCalled();
  });

  it("shows why a session failed, where its controls are", () => {
    // The panel's last duty, moved here with it.
    render(
      <DebugStrip
        {...base({ status: "failed", message: "no debug adapter for python on this machine." })}
      />,
    );
    expect(screen.getByRole("alert").textContent).toContain("no debug adapter");
  });

  it("installs a missing debugger and resumes without another click", async () => {
    // Protects docs/guarantees/debugging/a-missing-debugger-is-a-button.md
    //
    // The failure a person CAN fix from here. The old strip printed the whole
    // sentence — "…Install one with `hick dap install python`, or…" — and
    // truncated it at the strip's width, so the actionable half was off the
    // end of the control that could have done it.
    const onInstall = vi.fn().mockResolvedValue(undefined);
    const onStart = vi.fn();
    render(
      <DebugStrip
        {...base({
          status: "failed",
          message: "no debug adapter for python on this machine. Install one with…",
          offerInstall: { kind: "dap", language: "python" },
          onInstall,
          onStart,
        })}
      />,
    );
    expect(screen.queryByRole("button", { name: "Install it" })).toBeNull();
    await waitFor(() => expect(onInstall).toHaveBeenCalledWith({ kind: "dap", language: "python" }));
    await waitFor(() => expect(onStart).toHaveBeenCalledOnce());
  });

  it("still says the whole sentence when nothing here can install it", () => {
    // Go's adapter comes from `go install`. There is no button to offer, so
    // the prose is all there is — and it must not be swallowed.
    render(
      <DebugStrip
        {...base({
          status: "failed",
          message: "no debug adapter for go on this machine. delve is the Go debugger: …",
        })}
      />,
    );
    expect(screen.getByRole("alert").textContent).toContain("delve");
  });

  it("says how a finished program ended, and offers the way back in", () => {
    // The session behind a finished strip no longer exists — the server
    // reaped it with the program — so the strip's job is the exit info, a
    // re-run, and a dismissal. Not a row of step buttons for a dead session.
    const onStart = vi.fn();
    const onStop = vi.fn();
    render(
      <DebugStrip
        {...base({ status: "finished", exitCode: 0, frames: [], selectedFrame: null, onStart, onStop })}
      />,
    );
    expect(screen.getByRole("status").textContent).toContain("finished — exit code 0");
    // Stepping verbs are gone: there is nothing they could act on.
    expect(screen.queryByLabelText("Continue")).toBeNull();
    expect(screen.queryByLabelText("Step over")).toBeNull();
    // The re-run, with the breakpoints kept locally for it.
    fireEvent.click(screen.getByLabelText("Debug again"));
    expect(onStart).toHaveBeenCalled();
    // And Stop has become a dismissal of chrome, not of a session.
    expect(screen.queryByLabelText("Stop")).toBeNull();
    fireEvent.click(screen.getByLabelText("Dismiss"));
    expect(onStop).toHaveBeenCalled();
  });

  it("shows a failing exit code, and plain 'finished' when none was reported", () => {
    const { rerender } = render(
      <DebugStrip {...base({ status: "finished", exitCode: 3, frames: [], selectedFrame: null })} />,
    );
    expect(screen.getByRole("status").textContent).toContain("finished — exit code 3");
    rerender(
      <DebugStrip {...base({ status: "finished", exitCode: null, frames: [], selectedFrame: null })} />,
    );
    expect(screen.getByRole("status").textContent).toContain("finished");
    expect(screen.getByRole("status").textContent).not.toContain("exit code");
  });

  it("lists watches and lets each be removed", () => {
    const onRemoveWatch = vi.fn();
    render(
      <DebugStrip
        {...base({ watches: [{ expression: "quantity", value: "2" }], onRemoveWatch })}
      />,
    );
    fireEvent.click(screen.getByLabelText("Stop watching quantity"));
    expect(onRemoveWatch).toHaveBeenCalledWith("quantity");
  });
});

// Protects docs/guarantees/debugging/an-exception-breakpoint-uses-the-adapters-own-filters.md
describe("stopping on exceptions", () => {
  afterEach(cleanup);

  const caps = (filters: { id: string; label: string }[]) => ({
    conditional_breakpoints: true,
    hit_conditional_breakpoints: true,
    log_points: true,
    set_variable: true,
    restart_frame: false,
    step_in_targets: true,
    goto_targets: true,
    step_back: false,
    exception_filters: filters,
  });

  it("offers the adapter's own filters, in the adapter's own words", () => {
    render(
      <DebugStrip
        {...base({
          capabilities: caps([
            { id: "raised", label: "Raised Exceptions" },
            { id: "uncaught", label: "Uncaught Exceptions" },
          ]),
        })}
      />,
    );
    fireEvent.click(screen.getByLabelText("Debug options"));
    const switches = screen.getAllByRole("switch");
    expect(switches.map((s) => s.textContent)).toEqual([
      "Raised Exceptions",
      "Uncaught Exceptions",
    ]);
    expect(switches.every((s) => s.getAttribute("aria-checked") === "false")).toBe(true);
  });

  it("draws nothing for an adapter that has none", () => {
    render(<DebugStrip {...base({ capabilities: caps([]) })} />);
    expect(screen.queryAllByRole("switch")).toHaveLength(0);
  });

  it("shows which are on, and toggles by id", () => {
    const toggled: string[] = [];
    render(
      <DebugStrip
        {...base({
          capabilities: caps([
            { id: "raised", label: "Raised Exceptions" },
            { id: "uncaught", label: "Uncaught Exceptions" },
          ]),
          exceptionFilters: ["uncaught"],
          onToggleExceptionFilter: (id) => toggled.push(id),
        })}
      />,
    );
    fireEvent.click(screen.getByLabelText("Debug options"));
    const on = screen.getByRole("switch", { name: "Uncaught Exceptions" });
    expect(on.getAttribute("aria-checked")).toBe("true");
    fireEvent.click(screen.getByRole("switch", { name: "Raised Exceptions" }));
    // The id, never the label: the label is what a person reads and the id is
    // what the adapter answers to.
    expect(toggled).toEqual(["raised"]);
  });
});

describe("automatic debugger setup", () => {
  afterEach(cleanup);
  const offerInstall = { kind: "dap", language: "python" };
  it("reports installation failure and retries only when requested", async () => {
    const onInstall = vi.fn().mockRejectedValueOnce(new Error("Download failed")).mockResolvedValueOnce(undefined);
    const onStart = vi.fn();
    render(<DebugStrip {...base({ status: "failed", offerInstall, onInstall, onStart })} />);
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("Download failed"));
    expect(onInstall).toHaveBeenCalledTimes(1);
    expect(onStart).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Retry installation" }));
    await waitFor(() => expect(onStart).toHaveBeenCalledOnce());
    expect(onInstall).toHaveBeenCalledTimes(2);
  });
  it("does not start a session after setup is dismissed", async () => {
    let finish!: () => void;
    const pending = new Promise<void>(resolve => { finish = resolve; });
    const onInstall = vi.fn(() => pending);
    const onStart = vi.fn();
    const { rerender } = render(<DebugStrip {...base({ status: "failed", offerInstall, onInstall, onStart })} />);
    await waitFor(() => expect(onInstall).toHaveBeenCalledOnce());
    rerender(<DebugStrip {...base({ status: "idle", onStart })} />);
    finish();
    await pending;
    expect(onStart).not.toHaveBeenCalled();
  });
  it("does not loop installations if starting still reports a missing adapter", async () => {
    const onInstall = vi.fn().mockResolvedValue(undefined);
    const onStart = vi.fn();
    const { rerender } = render(<DebugStrip {...base({ status: "failed", offerInstall, onInstall, onStart })} />);
    await waitFor(() => expect(onStart).toHaveBeenCalledOnce());
    rerender(<DebugStrip {...base({ status: "starting", onInstall, onStart })} />);
    rerender(<DebugStrip {...base({ status: "failed", offerInstall, onInstall, onStart })} />);
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("still unavailable"));
    expect(onInstall).toHaveBeenCalledOnce();
  });
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares actual installer retry and refuses to restart a replacement target after a late installation", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let release!: () => void;
  const install = vi.fn().mockRejectedValueOnce(Error("first download failed")).mockImplementationOnce(() => new Promise<void>(resolve => { release = resolve; }));
  const start = vi.fn();
  const ui = (program: string) => <UiActionContext.Provider value={registry}><DebugStrip {...base({ program, status: "failed", offerInstall: { kind: "dap", language: "python" }, onInstall: install, onStart: start })} /></UiActionContext.Provider>;
  const rendered = render(ui("first.py"));
  const action = () => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === "Retry python debugger installation")!;
  try {
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("first download failed"));
    const descriptor = action(); let pending!: ReturnType<typeof registry.act>;
    await act(async () => { pending = registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: descriptor.handle, arguments: {}, request_id: "retry" }); });
    expect(install).toHaveBeenCalledTimes(2); expect(start).not.toHaveBeenCalled();
    rendered.rerender(ui("second.py"));
    await act(async () => { release(); expect(await pending).toMatchObject({ outcome: { status: "unavailable", reason: expect.stringContaining("source/setup owner") } }); });
    expect(start).not.toHaveBeenCalled();
    expect(screen.getByRole("alert").textContent).toContain("no debugger restart was requested");
    expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: descriptor.handle, arguments: {}, request_id: "old-target" })).toMatchObject({ status: "unavailable" });
    install.mockResolvedValue(undefined);
    await act(async () => expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: action().handle, arguments: {}, request_id: "current-target" })).toMatchObject({ outcome: { value: { installation_completed: true, debugger_restart_requested: true, running_debuggee: "not established" } } }));
    expect(start).toHaveBeenCalledTimes(1);
  } finally { rendered.unmount(); }
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("cancels the originating setup request without claiming to undo a shared download", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let release!: () => void;
  const install = vi.fn().mockRejectedValueOnce(Error("download failed")).mockImplementationOnce(() => new Promise<void>(resolve => { release = resolve; }));
  const start = vi.fn();
  const rendered = render(<UiActionContext.Provider value={registry}><DebugStrip {...base({ status: "failed", offerInstall: { kind: "dap", language: "python" }, onInstall: install, onStart: start })} /></UiActionContext.Provider>);
  try {
    await waitFor(() => expect(screen.getByRole("alert").textContent).toContain("download failed"));
    const descriptor = registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === "Retry python debugger installation")!;
    const actor = { id: "agent", kind: "agent" as const }; let pending!: ReturnType<typeof registry.act>;
    await act(async () => { pending = registry.act(registry.scope, actor, { handle: descriptor.handle, arguments: {}, request_id: "cancel-setup" }); });
    expect(registry.cancel(registry.scope, actor, "cancel-setup")).toBe(true);
    await act(async () => { release(); expect(await pending).toMatchObject({ outcome: { status: "unavailable", reason: expect.stringContaining("no debugger restart") } }); });
    expect(start).not.toHaveBeenCalled(); expect(install).toHaveBeenCalledTimes(2);
    expect(screen.getByRole("alert").textContent).toContain("Installation may have completed");
  } finally { rendered.unmount(); }
});
</book:entry>

</book:doc>
