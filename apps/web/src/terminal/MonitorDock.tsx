// The dock along the bottom: things that run so you can work, not things you
// are working on.
//
// A dev server has output all day and needs you approximately never. Giving
// it a tab means it competes with your work for space; giving it a place in
// the attention queue means the queue stops meaning anything. So it gets a
// strip that is always visible, never focused, and goes amber when something
// it was holding up has fallen over.

import type { TerminalSession } from "../api/types";
import { dockTone, monitors } from "../lib/monitorDock";

export function MonitorDock({
  sessions,
  onOpen,
  onClose,
}: {
  sessions: readonly TerminalSession[];
  onOpen: (id: string) => void;
  onClose: (id: string) => void;
}) {
  const shown = monitors(sessions);
  if (shown.length === 0) return null;
  const tone = dockTone(sessions);

  return (
    <div className={`monitor-dock monitor-dock-${tone}`} aria-label="Monitors">
      {shown.map((session) => (
        <div className="monitor-chip" key={session.id}>
          <span className={`state-dot state-${session.state}`} aria-hidden="true" />
          {/* The whole chip opens it, but opening is always the person's
              move: a monitor never pulls focus by itself, however loudly it
              fails. */}
          <button className="monitor-name" onClick={() => onOpen(session.id)}>
            {session.title}
          </button>
          <span className="monitor-preview" data-tip={session.preview}>
            {session.preview}
          </span>
          <button
            className="monitor-close"
            aria-label={`Stop ${session.title}`}
            onClick={() => onClose(session.id)}
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}
