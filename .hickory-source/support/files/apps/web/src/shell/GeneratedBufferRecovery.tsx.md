<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/GeneratedBufferRecovery.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3492f2559a3fce580696dbc491c49be6a72ed297d939e8d08cd0cb4da38ca917`.

<book:entry path="apps/web/src/shell/GeneratedBufferRecovery.tsx">
import { useId, useState } from "react";
import { ActionNotice } from "../components/ActionNotice";
import { viewBuffers } from "../lib/viewBuffers";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { OutputSaver } from "../lib/outputSave";
import type { UiOutcome } from "../lib/uiActions";
const owners = new WeakMap<object, string>();
function identity(owner: object | undefined) { if (!owner) return null; let id = owners.get(owner); if (!id) { id = crypto.randomUUID(); owners.set(owner, id); } return id; }
/** Recovery stays with the exact failed generated buffer and ordinary saver. */
export function GeneratedBufferRecovery({ bufferId, saver, error, isCurrent, source }: {
  bufferId: string; saver: OutputSaver; error: string; isCurrent: () => boolean; source: () => unknown;
}) {
  const parent = `generated-buffer-recovery:${useId()}`;
  const [failure, setFailure] = useState("");
  const revision = useUiRevision(() => [bufferId, identity(saver), identity(viewBuffers.get(bufferId)), error, source(), saver.baselineContent(), isCurrent(), failure]);
  const invoke = useUiCommands((["retry", "discard"] as const).map(operation => ({ key: `${parent}:${operation}`, command: {
    label: operation === "retry" ? "Retry generated buffer save" : "Discard generated buffer and reload", parent, subject: parent, parameters: {}, prerequisites: ["The current failed generated buffer; discard removes its unsaved changes through the existing recovery handler"],
    read: () => ({ revision: revision(), enabled: isCurrent() && !!viewBuffers.get(bufferId), reason: "The failed buffer owner changed or was closed" }),
    execute: async (): Promise<UiOutcome> => {
      const buffer = viewBuffers.get(bufferId);
      if (!buffer || !isCurrent()) return { status: "unavailable", reason: "The failed generated buffer is no longer current" };
      if (operation === "retry") {
        const saved = await saver.flushNow();
        if (!isCurrent()) return { status: "unavailable", reason: "The buffer owner changed during save; earlier effects may have occurred" };
        return { status: "completed", value: { saved, verification: "not requested" } };
      }
      await buffer.discard();
      if (!isCurrent()) return { status: "unavailable", reason: "The buffer owner changed during discard; earlier effects may have occurred" };
      return { status: "completed", value: { discarded: true, verification: "not requested" } };
    },
  } })));
  const act = (operation: string) => { void invoke(`${parent}:${operation}`).catch(cause => setFailure(cause instanceof Error ? cause.message : String(cause))); };
  return <ActionNotice kind="fail">
    Could not save into the owning document. Your edits remain in this buffer: {error}
    <button type="button" onClick={() => act("retry")}>Retry Save</button>
    <button type="button" onClick={() => act("discard")}>Discard this buffer and reload</button>
    {failure && <span role="alert">{failure}</span>}
  </ActionNotice>;
}
</book:entry>

</book:doc>
