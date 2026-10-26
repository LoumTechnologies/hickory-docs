<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An embed has no implicit host

For: Host developers embedding the document editor without a native execution service.

Given a host-controlled `DocumentEmbed` supplied with source, revision and path,
when it mounts, edits, receives host updates, or unmounts,
then it opens no API connection, socket, executor or analytics client. Its parser
loads as a static WASM asset. Changes carry the host's base revision; a host-driven
replacement is not echoed as a user edit. Two instances keep independent buffers,
registries, selections and disposal. Unsupported elements remain in the source.
Assets are resolved only through an explicit host callback.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/embedding/an-embed-has-no-implicit-host.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-dc7573551031"></a>
## `apps/web/src/embed/DocumentEmbed.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An embed has no implicit host](contract.md).

<a id="f-dc7573551031-Capability"></a>
### `Capability`

<storycode:copy id="f-dc7573551031-Capability">
import { AssetPreviewImage } from "./AssetPreviewImage";
import { useEffect, useState } from "react";
import type { EditorView } from "@codemirror/view";
import type { Extension } from "@codemirror/state";
import { EditingSurface } from "./EditingSurface";
import { loadHickLang } from "../editor/hickLang";
import "./embed.css";

export type Capability =
  | { state: "available" }
  | { state: "unsupported" | "denied" | "disconnected" | "missing"; reason: string };

</storycode:copy>
<a id="f-dc7573551031-EmbedHost"></a>
### `EmbedHost`

<storycode:copy id="f-dc7573551031-EmbedHost">
export interface EmbedHost {
  /** Explicitly supplied by the host; the editor never constructs API URLs. */
  resolveAsset?: (path: string, signal: AbortSignal) => Promise<string>;
  capabilities?: Readonly<Record<string, Capability>>;
  act?: (action: string, input: unknown) => Promise<unknown>;
}

</storycode:copy>
<a id="f-dc7573551031-DocumentEmbedProps"></a>
### `DocumentEmbedProps`

<storycode:copy id="f-dc7573551031-DocumentEmbedProps">
export interface DocumentEmbedProps {
  source: string;
  revision: string;
  path: string;
  host?: EmbedHost;
  readOnly?: boolean;
  onChange?: (change: { source: string; baseRevision: string; path: string }) => void;
  /** UTF-16 editor positions; engine provenance remains UTF-8 bytes. */
  onSelection?: (selection: { anchor: number; head: number }) => void;
  onViewReady?: (view: EditorView | null) => void;
  extensions?: Extension[];
}

</storycode:copy>
<a id="f-dc7573551031-DocumentEmbed"></a>
### `DocumentEmbed`

Host-controlled source. No room, socket, execution, autosave or analytics.

<storycode:copy id="f-dc7573551031-DocumentEmbed">
/** Host-controlled source. No room, socket, execution, autosave or analytics. */
export function DocumentEmbed(props: DocumentEmbedProps) {
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let disposed = false;
    void loadHickLang().then(() => { if (!disposed) setReady(true); })
      .catch((e: unknown) => { if (!disposed) setError(String(e)); });
    return () => { disposed = true; };
  }, []);
  return <div className="hickory-embed" data-revision={props.revision}>
    {error ? <p role="alert">Document parser could not load: {error}</p> : !ready ? <p role="status">Loading document parser…</p> :
      <EditingSurface value={props.source} syncToken={props.revision} hick readOnly={props.readOnly}
        ariaLabel={props.path} onSelection={props.onSelection} onViewReady={props.onViewReady}
        extraExtensions={props.extensions}
        onChange={(source) => props.onChange?.({ source, baseRevision: props.revision, path: props.path })} />}
    {props.host?.resolveAsset && <AssetPreview source={props.source} resolve={props.host.resolveAsset} />}
  </div>;
}

</storycode:copy>
<a id="f-dc7573551031-AssetPreview"></a>
### `AssetPreview`

<storycode:copy id="f-dc7573551031-AssetPreview">
function AssetPreview({ source, resolve }: { source: string; resolve: NonNullable<EmbedHost["resolveAsset"]> }) {
  const [assets, setAssets] = useState<{ path: string; url?: string; error?: string }[]>([]);
  useEffect(() => {
    const controller = new AbortController();
    const paths = [...new Set(Array.from(source.matchAll(/!\[[^\]]*\]\(([^\s)]+)\)/g), (m) => m[1]))];
    void Promise.all(paths.map(async (path) => {
      try { return { path, url: await resolve(path, controller.signal) }; }
      catch (e) { return { path, error: String(e) }; }
    })).then((next) => { if (!controller.signal.aborted) setAssets(next); });
    return () => controller.abort();
  }, [source, resolve]);
  return <div className="hickory-assets">{assets.map((asset) => asset.url
    ? <AssetPreviewImage key={`${asset.path}:${asset.url}`} path={asset.path} url={asset.url} />
    : <span role="alert" key={asset.path}>{asset.path}: {asset.error}</span>)}</div>;
}
</storycode:copy>

### Assemble `apps/web/src/embed/DocumentEmbed.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/DocumentEmbed.tsx"><storycode:paste select="#f-dc7573551031-Capability"/><storycode:paste select="#f-dc7573551031-EmbedHost"/><storycode:paste select="#f-dc7573551031-DocumentEmbedProps"/><storycode:paste select="#f-dc7573551031-DocumentEmbed"/><storycode:paste select="#f-dc7573551031-AssetPreview"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/embed/DocumentEmbed.tsx](contract.md#output-dc7573551031)
- [apps/web/src/embed/EditingSurface.tsx](../an-iframe-validates-its-host/contract.md#output-8378793ac20a)

### Tests and fixtures

- [apps/web/e2e/browserEmbedding.spec.ts](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#output-31019d3567c7)
- [apps/web/src/embed/DocumentEmbed.test.tsx](../../../mechanisms/apps/web/src/embed/DocumentEmbed.test.tsx.md#output-bb54f1265d97)
- [apps/web/src/embed/uiHostSession.test.tsx](../../../mechanisms/apps/web/src/embed/uiHostSession.ts.md#output-5f023c549320)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
