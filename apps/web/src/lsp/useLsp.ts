// One LSP session per open document, shared by both editors.
//
// The server owns the `hick-lsp` lifecycle, so this starts at `didOpen` with
// `hick:///<doc-path>` and keeps the server's copy in step with the editor
// through debounced full-text `didChange` (the bridge advertises FULL sync).
//
// Every language question is asked in DOCUMENT coordinates, including the ones
// that start in generated output: the bridge weaves the document into virtual
// files, delegates to the real language servers, and maps their answers back
// through provenance. Output-side callers convert their position to a source
// byte offset via provenance first (see lspPositionForOutput).

import { useEffect, useMemo, useRef, useState } from "react";
import { getWorkspaceRealtime, resetWorkspaceRealtime, type Realtime } from "../api/realtime";
import { LspClient, type LspDiagnostic } from "./client";

export interface LspSession {
  client: LspClient | null;
  uri: string;
  /** Diagnostics for this document, newest publish wins. */
  diagnostics: LspDiagnostic[];
  /** True once the session has an open document to ask questions about. */
  ready: boolean;
}

/** Milliseconds of typing quiet before the server is told the text changed. */
const SYNC_DEBOUNCE_MS = 300;

export function useLsp(realtime: Realtime, docPath: string, text: string): LspSession {
  const client = useMemo(() => {
    const channel = realtime.lsp();
    return channel ? new LspClient(channel) : null;
  }, [realtime]);
  // This session owns its client: the document's socket goes with the
  // document, and so does everything asked over it.
  useEffect(() => () => client?.dispose(), [client]);
  return useLspOver(client, docPath, text);
}

// The workspace's one client, shared by every plain file.
//
// One, not one per pane, for a reason that is not thrift: every client counts
// its request ids from 1, and two clients on ONE channel would each take the
// other's replies. Documents avoid this by each having a socket; plain files
// have no room and share the workspace socket, so they share the client too.
let workspaceClient: LspClient | null = null;
function workspaceLspClient(): LspClient | null {
  if (workspaceClient) return workspaceClient;
  const channel = getWorkspaceRealtime()?.lsp() ?? null;
  if (!channel) return null;
  workspaceClient = new LspClient(channel);
  return workspaceClient;
}

/** Test seam: forget the workspace client and the connection under it. */
export function resetWorkspaceLsp(): void {
  workspaceClient?.dispose();
  workspaceClient = null;
  resetWorkspaceRealtime();
}

/**
 * A language session for a plain file — `src/main.rs`, `app.py` — over the
 * workspace connection. The same questions, the same answers, at the file's
 * own path; the server treats a file that is not a document as its own
 * virtual file.
 */
export function useWorkspaceLsp(path: string, text: string): LspSession {
  const client = useMemo(() => workspaceLspClient(), []);
  return useLspOver(client, path, text);
}

function useLspOver(client: LspClient | null, docPath: string, text: string): LspSession {
  const uri = useMemo(() => `hick:///${docPath.replace(/^\/+/, "")}`, [docPath]);
  const [diagnostics, setDiagnostics] = useState<LspDiagnostic[]>([]);
  const [ready, setReady] = useState(false);
  const versionRef = useRef(1);
  const textRef = useRef(text);
  textRef.current = text;

  useEffect(() => {
    // No document path yet (the view is still loading): nothing to open.
    if (!client || !docPath) return;
    versionRef.current = 1;
    client.didOpen(uri, textRef.current, 1);
    setReady(true);
    const off = client.onDiagnostics((params) => {
      if (params.uri === uri) setDiagnostics(params.diagnostics);
    });
    return () => {
      off();
      client.didClose(uri);
      setReady(false);
    };
  }, [client, uri, docPath]);

  // Debounced full-text sync. Sending on every keystroke would make the bridge
  // re-weave the document per character.
  useEffect(() => {
    if (!client || !ready) return;
    const timer = setTimeout(() => {
      client.didChange(uri, text, ++versionRef.current);
    }, SYNC_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [client, ready, uri, text]);

  return { client, uri, diagnostics, ready };
}
