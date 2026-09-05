// Whether `.hick` documents merge through hick — asked once, at project open.
//
// The sharp edge this exists for: `.gitattributes` routing (`*.hick
// merge=hick`) is committed and reaches every clone, but the driver
// DEFINITION cannot be — git will not let a repository hand a clone an
// executable command. A clone that never ran `hick init` therefore merges
// `.hick` files with git's line merge, **silently**: no warning, no marker,
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

import { useEffect, useState } from "react";

import { api } from "../api/client";
import type { MergeDriverStatus } from "../api/types";

/** Dismissal is per window, not persisted: this is a fact about the clone,
 * and a person who dismissed it last month in another checkout has not fixed
 * anything here. */
export function MergeDriverNotice() {
  const [status, setStatus] = useState<MergeDriverStatus | null>(null);
  const [dismissed, setDismissed] = useState(false);
  // The button's own state: running, what it did, or why it could not.
  const [running, setRunning] = useState(false);
  const [done, setDone] = useState<string | null>(null);
  const [failed, setFailed] = useState<string | null>(null);

  // A fixable failure is a button, never a command to go and type
  // (docs/guarantees/collaboration/a-missing-merge-driver-is-a-button.md).
  // This runs the same `hick init` the sentence names, in the engine.
  const runInit = () => {
    setRunning(true);
    setFailed(null);
    api.initRepository().then(
      (outcome) => {
        setRunning(false);
        const wrote = Object.entries(outcome.changed)
          .filter(([, changed]) => changed)
          .map(([what]) => what.replace("_", "."));
        setDone(
          outcome.ok
            ? `hick init ran: ${wrote.length ? wrote.join(", ") : "everything was already in place"}. \`.hick\` documents now merge through hick in this clone.`
            : `hick init ran, but the driver is still not defined: ${outcome.status.summary}`,
        );
      },
      (e: unknown) => {
        setRunning(false);
        setFailed(e instanceof Error ? e.message : String(e));
      },
    );
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

  if (!status || dismissed) return null;
  if (done) {
    return (
      <div className="banner" role="status">
        <span>{done}</span>
        <button type="button" className="banner__dismiss" onClick={() => setDismissed(true)}>
          Dismiss
        </button>
      </div>
    );
  }
  return (
    <div className="banner banner-warn" role="status">
      <span>{failed ? `${status.summary} — hick init failed: ${failed}` : status.summary}</span>
      <button type="button" className="banner__action" onClick={runInit} disabled={running}>
        {running ? "Running hick init…" : "Run hick init"}
      </button>
      <button
        type="button"
        className="banner__dismiss"
        onClick={() => setDismissed(true)}
      >
        Dismiss
      </button>
    </div>
  );
}
