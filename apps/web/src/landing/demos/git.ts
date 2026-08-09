// The git state the collaboration demo shows, as a pure reducer.
//
// A demo that lies about git is worse than no demo: "Push" has to be
// impossible with nothing to push, "Pull" has to bring something that was not
// there before, and an uncommitted edit has to keep saying so until it is
// committed. Keeping the rules here — with no React and no clock — is what
// makes them assertable (see git.test.ts).

export interface Commit {
  /** Short hash. Derived from the message and its position, never random:
   * a demo whose ids change on re-render is a demo you cannot screenshot. */
  sha: string;
  message: string;
  author: string;
}

export interface GitState {
  commits: Commit[];
  /** How many of `commits` the remote has. Never exceeds commits.length. */
  pushed: number;
  /** Edits exist that no commit contains. */
  dirty: boolean;
}

export const INITIAL_GIT: GitState = {
  commits: [{ sha: "a1c9f30", message: "Add the plan limits document", author: "you" }],
  pushed: 1,
  dirty: false,
};

/** Deterministic short hash: enough to look like git, stable across renders. */
export function shaOf(message: string, index: number): string {
  let h = 0x811c9dc5;
  for (const ch of `${index}:${message}`) {
    h ^= ch.charCodeAt(0);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h.toString(16).padStart(8, "0").slice(0, 7);
}

export type GitAction =
  | { type: "edited" }
  | { type: "commit"; message: string; author: string }
  | { type: "push" }
  /**
   * `dirtyBefore` is the working-tree state as it was BEFORE the pulled text
   * landed. Applying a remote commit necessarily changes the document, which
   * would otherwise mark the tree dirty with nothing of yours in it — while
   * still not being allowed to swallow uncommitted work you already had.
   */
  | { type: "pull"; message: string; author: string; dirtyBefore: boolean };

export function gitReducer(state: GitState, action: GitAction): GitState {
  switch (action.type) {
    case "edited":
      return state.dirty ? state : { ...state, dirty: true };
    case "commit": {
      if (!state.dirty) return state;
      const message = action.message.trim() || "Update the document";
      const commits = [
        ...state.commits,
        { sha: shaOf(message, state.commits.length), message, author: action.author },
      ];
      return { commits, pushed: state.pushed, dirty: false };
    }
    case "push":
      return state.pushed === state.commits.length
        ? state
        : { ...state, pushed: state.commits.length };
    case "pull": {
      // A pulled commit is already on the remote, so it counts as pushed —
      // but only if everything before it was.
      const commits = [
        ...state.commits,
        { sha: shaOf(action.message, state.commits.length), message: action.message, author: action.author },
      ];
      const pushed = state.pushed === state.commits.length ? commits.length : state.pushed;
      return { commits, pushed, dirty: action.dirtyBefore };
    }
  }
}

/** What the Push button should say, and whether it can do anything. */
export function pushState(state: GitState): { label: string; enabled: boolean } {
  const ahead = state.commits.length - state.pushed;
  if (ahead <= 0) return { label: "Push", enabled: false };
  return { label: `Push (${ahead})`, enabled: true };
}
