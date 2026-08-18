// Sessions, grouped by the project they belong to — and what a folded group
// still tells you.
//
// The reason a group has a state at all: with several projects open, most
// rows are folded most of the time. If folding hid the fact that an agent is
// blocked, you would have to unfold everything to find out, which is the
// checking-on-things the queue exists to end. So a folded project shows the
// strongest claim any session under it is making.

import type { SessionState, TerminalSession } from "../api/types";

export interface SessionGroup {
  /** The working directory the sessions share — the project. */
  cwd: string;
  /** Its last path segment, which is what the row says. */
  name: string;
  sessions: TerminalSession[];
  /** The strongest claim under this group, shown while it is folded. */
  state: SessionState;
}

/** Most-claiming first. Working outranks idle: something is happening. */
const SEVERITY: Record<SessionState, number> = {
  "needs-you": 0,
  failed: 1,
  finished: 2,
  working: 3,
  idle: 4,
};

/** The strongest claim in a set of sessions. */
export function worstState(sessions: readonly TerminalSession[]): SessionState {
  return sessions.reduce<SessionState>(
    (worst, s) => (SEVERITY[s.state] < SEVERITY[worst] ? s.state : worst),
    "idle",
  );
}

/**
 * Group sessions by directory, in first-appearance order.
 *
 * Monitors are left out: they belong to the dock, and a dev server sitting in
 * the project tree would make every project look busy forever.
 */
export function groupSessions(sessions: readonly TerminalSession[]): SessionGroup[] {
  const groups: SessionGroup[] = [];
  for (const session of sessions) {
    if (session.monitor) continue;
    let group = groups.find((g) => g.cwd === session.cwd);
    if (!group) {
      group = {
        cwd: session.cwd,
        name: session.cwd.split(/[/\\]/).filter(Boolean).pop() ?? session.cwd,
        sessions: [],
        state: "idle",
      };
      groups.push(group);
    }
    group.sessions.push(session);
  }
  for (const group of groups) group.state = worstState(group.sessions);
  return groups;
}
