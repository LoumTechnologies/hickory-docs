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
  return (
    <div className="banner banner-warn" role="status">
      <span>{status.summary}</span>
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
