<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/uiDocumentRunActions.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-baf2c4dd123e"></a>
## `apps/web/src/views/uiDocumentRunActions.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-baf2c4dd123e-digest"></a>
### `digest`

<storycode:copy id="f-baf2c4dd123e-digest">
import { useId, useRef } from "react";
import { api, request } from "../api/client";
import { useUiCommands } from "../lib/uiActionBridge";
import type { UiValue } from "../lib/uiActions";

async function digest(text: string) {
  const bytes = new TextEncoder().encode(text);
  return [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))].map(byte => byte.toString(16).padStart(2, "0")).join("");
}
</storycode:copy>
<a id="f-baf2c4dd123e-useDocumentRunActions"></a>
### `useDocumentRunActions`

<storycode:copy id="f-baf2c4dd123e-useDocumentRunActions">
export function useDocumentRunActions(doc: string, current: () => string | null, lastRun: string | null, run: (expected?: string) => Promise<string | null>, verify: (expected?: string) => Promise<string | null>) {
  const instance = useId(); const parent = `document-execution:${doc}:${instance}`;
  const version = useRef<{ text: string | null; revision: string }>({ text: null, revision: crypto.randomUUID() });
  function revision() {
    const text = current();
    if (version.current.text !== text) version.current = { text, revision: crypto.randomUUID() };
    return `${version.current.revision}:${lastRun}`;
  }
  useUiCommands([
    ...[["run", "Run saved document", run], ["verify", "Verify saved document", verify]].map(([id, label, action]) => ({ key: `${parent}/${id}`, command: {
      label: label as string, subject: parent, parent, parameters: {}, prerequisites: ["Current editor bytes must match saved source; host binds execution to that SHA-256"],
      read: () => ({ revision: revision(), enabled: current() !== null, value: { document: doc, last_run: lastRun, verification: "Not established by a request" } }),
      execute: async () => {
        const text = current(); if (text === null) return { status: "unavailable" as const, reason: "Document editor unavailable" };
        const expected = await digest(text);
        const saved = await api.doc(doc);
        if (saved.source !== text || current() !== text) return { status: "unavailable" as const, reason: "Editor/source changed or has unsaved bytes; save and observe again" };
        const id = await (action as typeof run)(expected);
        return id ? { status: "queued" as const, job: id, value: { document: doc, purpose: label === "Verify saved document" ? "document-check" : "document-run", requested_source_hash: expected, result: "Read actual run receipt; queued does not mean passed" } }
          : { status: "unavailable" as const, reason: "Shared document controller could not start a run; inspect its banner" };
      },
    } })),
    { key: `${parent}/receipt`, command: {
      label: "Read latest document run receipt", subject: parent, parent, parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !!lastRun, reason: !lastRun ? "No run was started by this document controller" : undefined, value: lastRun }),
      execute: async () => {
        if (!lastRun) return { status: "unavailable", reason: "No run receipt" };
        const receipt = await request<Record<string, UiValue>>("GET", `/api/runs/${encodeURIComponent(lastRun)}`);
        const text = current(); const hash = text === null ? null : await digest(text);
        return { status: "completed", value: { receipt, current_editor_sha256: hash, root_source_matches_current_editor: typeof receipt.source_hash === "string" && receipt.source_hash.replace(/^sha256:/, "") === hash, scope: "Root source comparison at this read; inspect blocks, dependencies and errors before judging verification" } };
      },
    } },
  ]);
}
</storycode:copy>

### Assemble `apps/web/src/views/uiDocumentRunActions.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/uiDocumentRunActions.ts"><storycode:paste select="#f-baf2c4dd123e-digest"/><storycode:paste select="#f-baf2c4dd123e-useDocumentRunActions"/></storycode:file>

</storycode:doc>
