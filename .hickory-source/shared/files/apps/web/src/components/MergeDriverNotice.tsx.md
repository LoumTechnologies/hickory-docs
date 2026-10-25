<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/MergeDriverNotice.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Missing Merge Driver Is A Button, Not A Command To Go And Type](../../../../../../guarantees/collaboration/a-missing-merge-driver-is-a-button/contract.md)
- [`.md` Documents Merge Through Hick, And A Clone That Cannot Is Told](../../../../../../guarantees/collaboration/hick-documents-merge-through-hick/contract.md)

Captured SHA-256: `4685a72956902809ecd41967c28d092c1ff076fd8497ea4f35477cf537aec916`.

<book:entry path="apps/web/src/components/MergeDriverNotice.tsx">
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

function MergeDriverReading({ value }: { value: unknown }) { useUiReading("merge driver notice", value); return null; }
</book:entry>

</book:doc>
