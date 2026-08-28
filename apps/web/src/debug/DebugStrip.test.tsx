import { cleanup, fireEvent, render, screen } from "@testing-library/react";
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
});

describe("the function keys the tooltips promise", () => {
  it("maps F5, F10 and F11 to the steps their tooltips name", () => {
    expect(stepForKey("F5", false)).toBe("continue");
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

describe("the strip on the debugged block", () => {
  // Without cleanup, a strip from an earlier test keeps its key listener and
  // preventDefaults the F10 the next test fires.
  afterEach(cleanup);

  const base = (over: Partial<DebugStripProps> = {}): DebugStripProps => ({
    status: "paused",
    program: "orders.py",
    message: null,
    capabilities: null,
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

  it("offers to install a missing debugger instead of printing the command", () => {
    // Protects docs/guarantees/debugging/a-missing-debugger-is-a-button.md
    //
    // The failure a person CAN fix from here. The old strip printed the whole
    // sentence — "…Install one with `hick dap install python`, or…" — and
    // truncated it at the strip's width, so the actionable half was off the
    // end of the control that could have done it.
    const onInstall = vi.fn().mockResolvedValue(undefined);
    render(
      <DebugStrip
        {...base({
          status: "failed",
          message: "no debug adapter for python on this machine. Install one with…",
          offerInstall: { kind: "dap", language: "python" },
          onInstall,
        })}
      />,
    );
    expect(screen.getByRole("alert").textContent).toContain("No Python debugger");
    // And NOT the command: a button beside a command telling you to run the
    // command is the thing this replaces.
    expect(screen.getByRole("alert").textContent).not.toContain("hick dap install");
    fireEvent.click(screen.getByRole("button", { name: "Install it" }));
    expect(onInstall).toHaveBeenCalledWith({ kind: "dap", language: "python" });
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
