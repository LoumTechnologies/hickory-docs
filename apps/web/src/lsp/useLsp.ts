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
import type { Realtime } from "../api/realtime";
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
  const uri = useMemo(() => `hick:///${docPath.replace(/^\/+/, "")}`, [docPath]);
  const [diagnostics, setDiagnostics] = useState<LspDiagnostic[]>([]);
  const [ready, setReady] = useState(false);
  const clientRef = useRef<LspClient | null>(null);
  const versionRef = useRef(1);
  const textRef = useRef(text);
  textRef.current = text;

  const client = useMemo(() => {
    const channel = realtime.lsp();
    return channel ? new LspClient(channel) : null;
  }, [realtime]);
  clientRef.current = client;

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

  useEffect(() => () => client?.dispose(), [client]);

  return { client, uri, diagnostics, ready };
}
