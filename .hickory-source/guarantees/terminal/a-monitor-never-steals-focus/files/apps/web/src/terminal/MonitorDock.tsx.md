<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/terminal/MonitorDock.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Monitor Never Steals Focus](../../../../../contract.md)

Captured SHA-256: `9e8bd999776becc7a89f69b77935874cdd3a05ed685d1500c7e6cff52aa8f0e3`.

<book:entry path="apps/web/src/terminal/MonitorDock.tsx">
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
</book:entry>

</book:doc>
