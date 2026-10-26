<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Monitor Never Steals Focus

For: Authors running interactive tools alongside documents without losing session state.

Given a session opened with `monitor: true` — a dev server, a test watcher, a
log tail — when it prints, restarts, or dies, then it never enters the
attention queue, never opens a tab by itself, and never takes focus from
whatever pane is in front. It appears in the dock, where it shows its name and
its last line, and the dock turns amber when any monitor has stopped running.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/a-monitor-never-steals-focus.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-02bb99e89020"></a>
## `apps/web/src/lib/monitorDock.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Monitor Never Steals Focus](contract.md).

<a id="f-02bb99e89020-session"></a>
### `session`

<storycode:copy id="f-02bb99e89020-session">
import { describe, expect, it } from "vitest";

import type { TerminalSession } from "../api/types";
import { dockTone, monitors } from "./monitorDock";

function session(over: Partial<TerminalSession>): TerminalSession {
  return {
    id: "t1",
    title: "task",
    cwd: "/w",
    monitor: false,
    state: "working",
    since_ms: 0,
    branch: null,
    dirty: false,
    preview: "",
    prompt: null,
    exit_code: null,
    ...over,
  };
}

</storycode:copy>
<a id="f-02bb99e89020-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-02bb99e89020-expression_statement">
describe("the monitor dock", () => {
  it("holds only the monitors", () => {
    const all = [
      session({ id: "task" }),
      session({ id: "server", monitor: true }),
    ];
    expect(monitors(all).map((s) => s.id)).toEqual(["server"]);
  });

  it("stays quiet while its processes are running", () => {
    expect(dockTone([session({ id: "server", monitor: true, state: "working" })])).toBe(
      "quiet",
    );
  });

  it("goes amber when a monitor dies, however it died", () => {
    expect(dockTone([session({ monitor: true, state: "failed" })])).toBe("amber");
    expect(dockTone([session({ monitor: true, state: "finished" })])).toBe("amber");
  });

  it("ignores ordinary sessions — a failed task belongs in the queue, not the dock", () => {
    expect(dockTone([session({ monitor: false, state: "failed" })])).toBe("quiet");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/monitorDock.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/monitorDock.test.ts"><storycode:paste select="#f-02bb99e89020-session"/><storycode:paste select="#f-02bb99e89020-expression_statement"/></storycode:file>

<a id="output-c298f7b894fe"></a>
## `apps/web/src/lib/monitorDock.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Monitor Never Steals Focus](contract.md).

<a id="f-c298f7b894fe-DockTone"></a>
### `DockTone`

The monitor dock's one decision: how loud to be.
Dev servers, watchers, and log tails are long-running support processes.
They are always saying something, so they must never claim attention the
way a task does — a queue that fills with "webpack recompiled" is a queue
people stop reading. But a dev server that has actually fallen over is
worth noticing, so the dock goes amber and waits to be looked at, rather
than interrupting.
Protects docs/guarantees/terminal/a-monitor-never-steals-focus.md

<storycode:copy id="f-c298f7b894fe-DockTone">
// The monitor dock's one decision: how loud to be.
//
// Dev servers, watchers, and log tails are long-running support processes.
// They are always saying something, so they must never claim attention the
// way a task does — a queue that fills with "webpack recompiled" is a queue
// people stop reading. But a dev server that has actually fallen over is
// worth noticing, so the dock goes amber and waits to be looked at, rather
// than interrupting.
//
// Protects docs/guarantees/terminal/a-monitor-never-steals-focus.md

import type { TerminalSession } from "../api/types";

export type DockTone = "quiet" | "amber";

</storycode:copy>
<a id="f-c298f7b894fe-monitors"></a>
### `monitors`

<storycode:copy id="f-c298f7b894fe-monitors">
/** Only the sessions that belong in the dock. */
export function monitors(sessions: readonly TerminalSession[]): TerminalSession[] {
  return sessions.filter((s) => s.monitor);
}

</storycode:copy>
<a id="f-c298f7b894fe-dockTone"></a>
### `dockTone`

How the dock should look. Amber when any monitor has died — failed, or
exited at all: a dev server that ended cleanly still ended, and the thing
you were relying on is not there any more.

<storycode:copy id="f-c298f7b894fe-dockTone">
/**
 * How the dock should look. Amber when any monitor has died — failed, or
 * exited at all: a dev server that ended cleanly still ended, and the thing
 * you were relying on is not there any more.
 */
export function dockTone(sessions: readonly TerminalSession[]): DockTone {
  const dead = monitors(sessions).some(
    (s) => s.state === "failed" || s.state === "finished",
  );
  return dead ? "amber" : "quiet";
}
</storycode:copy>

### Assemble `apps/web/src/lib/monitorDock.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/monitorDock.ts"><storycode:paste select="#f-c298f7b894fe-DockTone"/><storycode:paste select="#f-c298f7b894fe-monitors"/><storycode:paste select="#f-c298f7b894fe-dockTone"/></storycode:file>

<a id="output-9ef92b071e30"></a>
## `apps/web/src/terminal/MonitorDock.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Monitor Never Steals Focus](contract.md).

<a id="f-9ef92b071e30-MonitorDock"></a>
### `MonitorDock`

The dock along the bottom: things that run so you can work, not things you
are working on.
A dev server has output all day and needs you approximately never. Giving
it a tab means it competes with your work for space; giving it a place in
the attention queue means the queue stops meaning anything. So it gets a
strip that is always visible, never focused, and goes amber when something
it was holding up has fallen over.

<storycode:copy id="f-9ef92b071e30-MonitorDock">
// The dock along the bottom: things that run so you can work, not things you
// are working on.
//
// A dev server has output all day and needs you approximately never. Giving
// it a tab means it competes with your work for space; giving it a place in
// the attention queue means the queue stops meaning anything. So it gets a
// strip that is always visible, never focused, and goes amber when something
// it was holding up has fallen over.

import { useTerminalChromeActions } from "./useTerminalChromeActions";
import type { TerminalSession } from "../api/types";
import { dockTone, monitors } from "../lib/monitorDock";

export function MonitorDock({
  sessions,
  onOpen,
  onClose,
}: {
  sessions: readonly TerminalSession[];
  onOpen: (id: string) => boolean | void | Promise<boolean | void>;
  onClose: (id: string) => boolean | void | Promise<boolean | void>;
}) {
  const shown = monitors(sessions);
  const controls = useTerminalChromeActions("terminal monitor dock", shown.map(session => ({ id: session.id, title: session.title, state: session.state, preview: session.preview })),
    shown.flatMap(session => [
      { id: `open:${session.id}`, label: `Open monitor terminal: ${session.title}`, run: () => onOpen(session.id) },
      { id: `close:${session.id}`, label: `Stop monitor terminal: ${session.title}`, run: () => onClose(session.id) },
    ]));
  if (shown.length === 0) return null;
  const tone = dockTone(sessions);

  return (
    <div className={`monitor-dock monitor-dock-${tone}`} aria-label="Monitors">
      {controls.error && <span role="alert">{controls.error}</span>}
      {shown.map((session) => (
        <div className="monitor-chip" key={session.id}>
          <span className={`state-dot state-${session.state}`} aria-hidden="true" />
          {/* The whole chip opens it, but opening is always the person's
              move: a monitor never pulls focus by itself, however loudly it
              fails. */}
          <button className="monitor-name" disabled={!!controls.pending} onClick={() => controls.human(`open:${session.id}`)}>
            {session.title}
          </button>
          <span className="monitor-preview" data-tip={session.preview}>
            {session.preview}
          </span>
          <button
            className="monitor-close"
            aria-label={`Stop ${session.title}`}
            disabled={!!controls.pending}
            onClick={() => controls.human(`close:${session.id}`)}
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/terminal/MonitorDock.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/terminal/MonitorDock.tsx"><storycode:paste select="#f-9ef92b071e30-MonitorDock"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/lib/monitorDock.ts](contract.md#output-c298f7b894fe)
- [apps/web/src/terminal/MonitorDock.tsx](contract.md#output-9ef92b071e30)

### Tests and fixtures

- [apps/web/src/lib/monitorDock.test.ts](contract.md#output-02bb99e89020)
- [crates/hickory-cli/tests/serve_terminals.rs](../../../mechanisms/crates/hickory-cli/tests/serve_terminals.rs.md#output-5d8946e53299)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: some-source-citations-do-not-resolve.

</storycode:doc>
