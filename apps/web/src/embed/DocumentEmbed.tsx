import { useEffect, useState } from "react";
import type { EditorView } from "@codemirror/view";
import type { Extension } from "@codemirror/state";
import { EditingSurface } from "./EditingSurface";
import { loadHickLang } from "../editor/hickLang";
import "./embed.css";

export type Capability =
  | { state: "available" }
  | { state: "unsupported" | "denied" | "disconnected" | "missing"; reason: string };

export interface EmbedHost {
  /** Explicitly supplied by the host; the editor never constructs API URLs. */
  resolveAsset?: (path: string, signal: AbortSignal) => Promise<string>;
  capabilities?: Readonly<Record<string, Capability>>;
  act?: (action: string, input: unknown) => Promise<unknown>;
}

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
    ? <img key={asset.path} src={asset.url} alt={asset.path} />
    : <span role="alert" key={asset.path}>{asset.path}: {asset.error}</span>)}</div>;
}
