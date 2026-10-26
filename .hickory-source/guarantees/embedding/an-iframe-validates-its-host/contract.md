<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An iframe validates its host

For: Host developers embedding the document editor without a native execution service.

Given a static document iframe configured with a concrete parent origin,
when a versioned embedding message arrives,
then it is accepted only from its actual parent window at that exact origin.
Load supplies source/revision and optional asset URLs; Save returns the current
source for the host to persist. Disposal unmounts the editor and terminates its
debug worker. No credentials are sent to wildcard origins. Storage remains the
host's responsibility when iframe-local persistence is unavailable.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/an-iframe-validates-its-host.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-8378793ac20a"></a>
## `apps/web/src/embed/EditingSurface.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An embed has no implicit host](../an-embed-has-no-implicit-host/contract.md), [An iframe validates its host](contract.md).

<a id="f-8378793ac20a-EditingSurfaceProps"></a>
### `EditingSurfaceProps`

<storycode:copy id="f-8378793ac20a-EditingSurfaceProps">
import { useEffect, useMemo, useRef, useState } from "react";
import { useEditorActions } from "../editor/uiEditorActions";
import { useEditorEditing } from "../editor/useEditorEditing";
import { Annotation, EditorState } from "@codemirror/state";
import type { Extension } from "@codemirror/state";
import { EditorView, keymap, drawSelection } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import type * as Y from "yjs";
import type { Awareness } from "y-protocols/awareness";
import { yCollab } from "y-codemirror.next";
import { foldEffect } from "@codemirror/language";
import { EnvRegistry, wysiwyg } from "../editor/wysiwyg";
import { rangeHighlightField } from "../editor/rangeHighlight";
import { foldRangesOf, hickoryFolding } from "../editor/folding";
import { languageExtensions } from "../editor/languages";

export interface EditingSurfaceProps {
  /** Buffer contents. Ignored when `collab` is set — the CRDT owns the text. */
  value: string;
  onChange?: (next: string) => void;
  onSelection?: (selection: { anchor: number; head: number }) => void;
  /** Render `.md` source with the product's document decorations. */
  hick?: boolean;
  /**
   * Hick block names to collapse on arrival, e.g. `["file"]`.
   *
   * A demo pane is a fraction of the height of the real workspace, and every
   * fragment scrolled out of it turns its lineage ribbon into a faded stub
   * clamped to the pane edge. Folding the bodies the visitor does not need to
   * read — the generated file's scaffolding, which they can see rendered on
   * the right — is what puts the whole flow on screen at once. It is the
   * product's own fold, not a demo-only trick: the chevrons work.
   */
  foldBlocks?: string[];
  /** Highlight a generated file's language instead. */
  language?: string;
  readOnly?: boolean;
  /** Live EditorView on mount, null on teardown (ribbon measurement). */
  onViewReady?: (view: EditorView | null) => void;
  /** Bind this pane to a CRDT room shared with another pane. */
  collab?: { ytext: Y.Text; awareness: Awareness };
  /**
   * Bump to force the buffer back to `value` when `value` itself did not
   * change. A rejected edit (one that touched weaver-generated text) leaves
   * the buffer holding text the document never accepted; without this the
   * pane would keep showing it, because the prop it is meant to mirror is
   * byte-for-byte what it was before the keystroke.
   */
  syncToken?: number | string;
  /**
   * Extra CodeMirror extensions, e.g. the language-server bindings.
   *
   * Passed in rather than built here so the demo that wants intelligence gets
   * it and the demos that do not are unaffected — and so the extensions it
   * gets are the product's own, not a demo-shaped imitation of them.
   */
  extraExtensions?: Extension[];
  className?: string;
  ariaLabel: string;
  testId?: string;
}

</storycode:copy>
<a id="f-8378793ac20a-EditingSurface"></a>
### `EditingSurface`

<storycode:copy id="f-8378793ac20a-EditingSurface">
export function EditingSurface({
  value,
  onChange,
  onSelection,
  hick,
  foldBlocks,
  language,
  readOnly,
  onViewReady,
  collab,
  syncToken,
  extraExtensions,
  className,
  ariaLabel,
  testId,
}: EditingSurfaceProps) {
  const [mounted, setMounted] = useState<EditorView | null>(null);
  useEditorActions(mounted, `embed:${ariaLabel}`);
  useEditorEditing(mounted, { source: `embed:${ariaLabel}`, label: "embedded document", identity: () => [ariaLabel, syncToken], current: () => !readOnly, localOnly: true });
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  // Held in a ref so a changing callback never tears down the editor.
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const selectionRef = useRef(onSelection);
  selectionRef.current = onSelection;
  const envRegistry = useMemo(() => new EnvRegistry(), []);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    const extensions: Extension[] = [
      history(),
      drawSelection(),
      keymap.of([...defaultKeymap, ...historyKeymap]),
      rangeHighlightField,
      EditorView.lineWrapping,
      EditorView.editable.of(!readOnly),
      EditorState.readOnly.of(!!readOnly),
      EditorView.updateListener.of((u) => {
        if (u.docChanged && !u.transactions.some((tr) => tr.annotation(hostUpdate))) onChangeRef.current?.(u.state.doc.toString());
        if (u.selectionSet) selectionRef.current?.(u.state.selection.main);
      }),
    ];
    if (hick) extensions.push(wysiwyg(envRegistry), hickoryFolding());
    else if (language) extensions.push(...languageExtensions(language));
    if (collab) extensions.push(yCollab(collab.ytext, collab.awareness));
    if (extraExtensions?.length) extensions.push(...extraExtensions);

    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: collab ? collab.ytext.toString() : value,
        extensions,
      }),
    });
    viewRef.current = view; setMounted(view);
    onViewReady?.(view);
    return () => {
      onViewReady?.(null);
      view.destroy();
      viewRef.current = null; setMounted(null);
    };
    // The buffer's identity — not its contents — decides when to rebuild.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hick, language, readOnly, collab, envRegistry, extraExtensions]);

  // Push an externally-driven value (a walkthrough step, or an edit mapped
  // back from the output pane) into the buffer. Under `collab` the CRDT is the
  // only writer; replacing the doc here would fight it and lose text.
  useEffect(() => {
    const view = viewRef.current;
    if (!view || collab) return;
    const current = view.state.doc.toString();
    if (current === value) return;
    const head = Math.min(view.state.selection.main.head, value.length);
    view.dispatch({
      annotations: hostUpdate.of(true),
      changes: { from: 0, to: current.length, insert: value },
      selection: { anchor: head },
    });
  }, [value, syncToken, collab]);

  // Fold the named blocks as they appear. Runs after the value sync above, so
  // a step that adds three file blocks folds all three.
  //
  // Each range is folded at most ONCE. Without that, unfolding a block to look
  // inside it would last exactly until the next keystroke re-folded it, and
  // the editor would be arguing with the person using it.
  const foldedOnceRef = useRef(new Set<string>());
  useEffect(() => {
    const view = viewRef.current;
    if (!view || !foldBlocks || foldBlocks.length === 0) return;
    const seen = foldedOnceRef.current;
    const effects = foldRangesOf(view.state)
      .filter((r) => r.kind === "block" && r.name && foldBlocks.includes(r.name))
      .filter((r) => !seen.has(`${r.from}:${r.to}`))
      .map((r) => {
        seen.add(`${r.from}:${r.to}`);
        return foldEffect.of({ from: r.from, to: r.to });
      });
    if (effects.length > 0) view.dispatch({ effects });
  }, [value, foldBlocks]);

  return (
    <div
      ref={hostRef}
      className={`demo-editor${hick ? " demo-editor-hick" : ""}${className ? ` ${className}` : ""}`}
      role="group"
      aria-label={ariaLabel}
      data-testid={testId}
    />
  );
}

</storycode:copy>
<a id="f-8378793ac20a-hostUpdate"></a>
### `hostUpdate`

<storycode:copy id="f-8378793ac20a-hostUpdate">
const hostUpdate = Annotation.define<boolean>();
</storycode:copy>

### Assemble `apps/web/src/embed/EditingSurface.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/EditingSurface.tsx"><storycode:paste select="#f-8378793ac20a-EditingSurfaceProps"/><storycode:paste select="#f-8378793ac20a-EditingSurface"/><storycode:paste select="#f-8378793ac20a-hostUpdate"/></storycode:file>

<a id="output-c52be491a72e"></a>
## `apps/web/src/embed/iframe.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An iframe validates its host](contract.md).

<a id="f-c52be491a72e-CHANNEL"></a>
### `CHANNEL`

<storycode:copy id="f-c52be491a72e-CHANNEL">
import { useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import { UiActionContext } from "../lib/uiActionBridge";
import { createEmbedUiSession } from "./uiHostSession";
import { DebuggableDocument } from "./DebuggableDocument";
import type { DocumentEmbedProps } from "./DocumentEmbed";

const CHANNEL = "hickory-embed";
</storycode:copy>
<a id="f-c52be491a72e-UiSession"></a>
### `UiSession`

<storycode:copy id="f-c52be491a72e-UiSession">
type UiSession = ReturnType<typeof createEmbedUiSession>;
</storycode:copy>
<a id="f-c52be491a72e-LoadedDocument"></a>
### `LoadedDocument`

<storycode:copy id="f-c52be491a72e-LoadedDocument">
interface LoadedDocument { ui: UiSession; path: string; source: string; revision: string; edit: number; readOnly: boolean; assets: Record<string, string>; }
</storycode:copy>
<a id="f-c52be491a72e-configured"></a>
### `configured`

<storycode:copy id="f-c52be491a72e-configured">
const configured = new URL(location.href).searchParams.get("parentOrigin");
</storycode:copy>
<a id="f-c52be491a72e-parentOrigin"></a>
### `parentOrigin`

<storycode:copy id="f-c52be491a72e-parentOrigin">
let parentOrigin: string | null = null;
</storycode:copy>
<a id="f-c52be491a72e-try_statement"></a>
### `try_statement`

<storycode:copy id="f-c52be491a72e-try_statement">
try { if (configured && new URL(configured).origin === configured && configured !== "null") parentOrigin = configured; } catch { /* rejected below */ }

</storycode:copy>
<a id="f-c52be491a72e-Frame"></a>
### `Frame`

<storycode:copy id="f-c52be491a72e-Frame">
function Frame() {
  const [document, setDocument] = useState<LoadedDocument | null>(null);
  const [disposed, setDisposed] = useState(false);
  const [error, setError] = useState<string | null>(null);
  function send(event: string, data: unknown = null, id?: string) {
    if (parentOrigin) window.parent.postMessage({ channel: CHANNEL, version: 1, event, data, id }, parentOrigin);
  }
  useEffect(() => {
    if (!parentOrigin || window.parent === window) return;
    let ended = false;
    let ui: UiSession | null = null;
    const listener = (event: MessageEvent) => {
      if (event.source !== window.parent || event.origin !== parentOrigin || ended) return;
      const message = event.data;
      if (!message || message.channel !== CHANNEL || message.version !== 1) return;
      try {
        switch (message.op) {
          case "load": {
            const data = message.data;
            if (!data || typeof data.source !== "string" || typeof data.path !== "string" || !data.path.endsWith(".md") || typeof data.revision !== "string") throw Error("load requires a .md path, source and revision");
            const assets: Record<string, string> = Object.create(null);
            for (const [path, url] of Object.entries(data.assets ?? {})) {
              if (typeof url !== "string" || !/^(https?:|blob:|data:image\/)/.test(url)) throw Error(`Invalid asset URL: ${path}`);
              assets[path] = url;
            }
            ui?.dispose();
            ui = createEmbedUiSession(value => send("ui-receipt", value));
            setDocument({ ui, path: data.path, source: data.source, revision: data.revision, edit: 0, readOnly: data.readOnly === true, assets });
            setError(null); send("loaded", { revision: data.revision, ui_scope: ui.scope }, message.id); break;
          }
          case "ui": {
            if (!ui) throw Error("Load a document before requesting its UI controls");
            if (typeof message.id !== "string" || !message.id.length || message.id.length > 128) throw Error("Embedded UI requests require a bounded response id");
            const currentUi = ui;
            void currentUi.invoke(message.data).then(result => send("ui-result", { scope: currentUi.scope, result }, message.id), failure => send("error", { message: String(failure), ui_scope: currentUi.scope }, message.id));
            break;
          }
          case "save":
            setDocument((current) => { if (current) send("save", { path: current.path, source: current.source, revision: current.revision }, message.id); return current; }); break;
          case "dispose": ui?.dispose(); ended = true; window.removeEventListener("message", listener); setDisposed(true); setDocument(null); send("disposed", null, message.id); break;
          default: throw Error(`Unsupported embedding operation: ${message.op}`);
        }
      } catch (failure) { setError(String(failure)); send("error", { message: String(failure) }, message.id); }
    };
    window.addEventListener("message", listener);
    send("ready", { capabilities: { editing: true, browserDebug: true, nativeExecution: false, uiActions: "scoped-v1", uiActor: "embed-parent agent; no human attestation", receiptStorage: "frame session results; host must persist receipt events", storage: "host" } });
    return () => { ended = true; ui?.dispose(); window.removeEventListener("message", listener); };
  }, []);
  const host = useMemo(() => ({ resolveAsset: async (path: string) => {
    const url = document?.assets[path]; if (!url) throw Error(`Host did not supply asset ${path}`); return url;
  } }), [document?.assets]);
  const change: DocumentEmbedProps["onChange"] = (next) => {
    setDocument((current) => current ? { ...current, source: next.source, edit: current.edit + 1 } : null);
    send("change", { ...next, baseRevision: document?.revision });
  };
  if (!parentOrigin) return <p role="alert">The host must configure a concrete parentOrigin.</p>;
  if (disposed) return null;
  return <>{error && <p role="alert">{error}</p>}{document
    ? <UiActionContext.Provider key={document.ui.scope.window} value={document.ui.registry}><DebuggableDocument path={document.path} source={document.source} revision={`${document.revision}:${document.edit}`} readOnly={document.readOnly} host={host}
        onChange={change} onSelection={(selection) => send("selection", selection)} /></UiActionContext.Provider>
    : <p role="status">Waiting for the host's document…</p>}</>;
}
</storycode:copy>
<a id="f-c52be491a72e-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c52be491a72e-expression_statement">
createRoot(document.getElementById("root")!).render(<Frame />);
</storycode:copy>

### Assemble `apps/web/src/embed/iframe.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/iframe.tsx"><storycode:paste select="#f-c52be491a72e-CHANNEL"/><storycode:paste select="#f-c52be491a72e-UiSession"/><storycode:paste select="#f-c52be491a72e-LoadedDocument"/><storycode:paste select="#f-c52be491a72e-configured"/><storycode:paste select="#f-c52be491a72e-parentOrigin"/><storycode:paste select="#f-c52be491a72e-try_statement"/><storycode:paste select="#f-c52be491a72e-Frame"/><storycode:paste select="#f-c52be491a72e-expression_statement"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/embed/EditingSurface.tsx](contract.md#output-8378793ac20a)
- [apps/web/src/embed/iframe.tsx](contract.md#output-c52be491a72e)
- [apps/web/src/embed/uiHostSession.ts](../../../mechanisms/apps/web/src/embed/uiHostSession.ts.md#output-585a885211aa)

### Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#output-31019d3567c7)
- [apps/web/src/embed/uiHostSession.test.tsx](../../../mechanisms/apps/web/src/embed/uiHostSession.ts.md#output-5f023c549320)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
