<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Plain File Has The Same Language Server A Document Has

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a folder open in the app and a text file in it that is not a `.md`
document — `src/main.rs`, `app.py`, `index.ts` — when it is opened in a
pane, then every language question the document editor can ask is asked
about this file too: diagnostics, hover, completion, go to definition, type
definition and implementation, find references, document highlight, inlay
hints, semantic colouring, signature help, folding, rename and code actions;
and the answers are in the file's own coordinates, at the file's own path,
with the folder the app opened as the project root. A definition that lands
in another file opens that file's tab at that line, and the file's problems
count in the status bar and appear in the Problems list.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/a-plain-file-has-the-same-language-server.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-50cf197029a3"></a>
## `apps/web/src/lib/fileProblems.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Plain File Has The Same Language Server A Document Has](contract.md).

<a id="f-50cf197029a3-problems"></a>
### `problems`

The problems in plain files, for the count and the list.
A document's diagnostics live on its session, and the status bar reads them
from the registry. A plain file has no session — its pane is the whole of
it — so its diagnostics are put here, keyed by path, and the workspace adds
them in. Set only from the pane that owns the file, cleared when it goes:
a count that includes a file nobody has open is a count that cannot be
expanded into anything.

<storycode:copy id="f-50cf197029a3-problems">
// The problems in plain files, for the count and the list.
//
// A document's diagnostics live on its session, and the status bar reads them
// from the registry. A plain file has no session — its pane is the whole of
// it — so its diagnostics are put here, keyed by path, and the workspace adds
// them in. Set only from the pane that owns the file, cleared when it goes:
// a count that includes a file nobody has open is a count that cannot be
// expanded into anything.

import { useSyncExternalStore } from "react";
import type { LspDiagnostic } from "../lsp/client";

const problems = new Map<string, LspDiagnostic[]>();
</storycode:copy>
<a id="f-50cf197029a3-listeners"></a>
### `listeners`

<storycode:copy id="f-50cf197029a3-listeners">
const listeners = new Set<() => void>();
</storycode:copy>
<a id="f-50cf197029a3-version"></a>
### `version`

<storycode:copy id="f-50cf197029a3-version">
let version = 0;

</storycode:copy>
<a id="f-50cf197029a3-bump"></a>
### `bump`

<storycode:copy id="f-50cf197029a3-bump">
function bump() {
  version++;
  for (const listener of listeners) listener();
}

</storycode:copy>
<a id="f-50cf197029a3-setFileProblems"></a>
### `setFileProblems`

Record what the server said about `path`; an empty list clears it.

<storycode:copy id="f-50cf197029a3-setFileProblems">
/** Record what the server said about `path`; an empty list clears it. */
export function setFileProblems(path: string, diagnostics: LspDiagnostic[]): void {
  if (diagnostics.length === 0) {
    if (!problems.delete(path)) return;
  } else {
    problems.set(path, diagnostics);
  }
  bump();
}

</storycode:copy>
<a id="f-50cf197029a3-forgetFileProblems"></a>
### `forgetFileProblems`

The pane closed: nothing about this file is known any more.

<storycode:copy id="f-50cf197029a3-forgetFileProblems">
/** The pane closed: nothing about this file is known any more. */
export function forgetFileProblems(path: string): void {
  if (problems.delete(path)) bump();
}

</storycode:copy>
<a id="f-50cf197029a3-allFileProblems"></a>
### `allFileProblems`

Every file with problems, for the workspace to add to its own.

<storycode:copy id="f-50cf197029a3-allFileProblems">
/** Every file with problems, for the workspace to add to its own. */
export function allFileProblems(): { path: string; diagnostics: LspDiagnostic[] }[] {
  return [...problems.entries()].map(([path, diagnostics]) => ({ path, diagnostics }));
}

</storycode:copy>
<a id="f-50cf197029a3-subscribe"></a>
### `subscribe`

<storycode:copy id="f-50cf197029a3-subscribe">
function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

</storycode:copy>
<a id="f-50cf197029a3-useFileProblemsVersion"></a>
### `useFileProblemsVersion`

A version that changes whenever any file's problems do.

<storycode:copy id="f-50cf197029a3-useFileProblemsVersion">
/** A version that changes whenever any file's problems do. */
export function useFileProblemsVersion(): number {
  return useSyncExternalStore(subscribe, () => version, () => version);
}

</storycode:copy>
<a id="f-50cf197029a3-resetFileProblems"></a>
### `resetFileProblems`

<storycode:copy id="f-50cf197029a3-resetFileProblems">
/** Test seam. */
export function resetFileProblems(): void {
  problems.clear();
  bump();
}
</storycode:copy>

### Assemble `apps/web/src/lib/fileProblems.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/fileProblems.ts"><storycode:paste select="#f-50cf197029a3-problems"/><storycode:paste select="#f-50cf197029a3-listeners"/><storycode:paste select="#f-50cf197029a3-version"/><storycode:paste select="#f-50cf197029a3-bump"/><storycode:paste select="#f-50cf197029a3-setFileProblems"/><storycode:paste select="#f-50cf197029a3-forgetFileProblems"/><storycode:paste select="#f-50cf197029a3-allFileProblems"/><storycode:paste select="#f-50cf197029a3-subscribe"/><storycode:paste select="#f-50cf197029a3-useFileProblemsVersion"/><storycode:paste select="#f-50cf197029a3-resetFileProblems"/></storycode:file>

<a id="output-5ad96644c888"></a>
## `apps/web/src/lsp/useLsp.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Plain File Has The Same Language Server A Document Has](contract.md).

<a id="f-5ad96644c888-LspSession"></a>
### `LspSession`

One LSP session per open document, shared by both editors.
The server owns the `hick-lsp` lifecycle, so this starts at `didOpen` with
`hick:///<doc-path>` and keeps the server's copy in step with the editor
through debounced full-text `didChange` (the bridge advertises FULL sync).
Every language question is asked in DOCUMENT coordinates, including the ones
that start in generated output: the bridge weaves the document into virtual
files, delegates to the real language servers, and maps their answers back
through provenance. Output-side callers convert their position to a source
byte offset via provenance first (see lspPositionForOutput).

<storycode:copy id="f-5ad96644c888-LspSession">
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
import { documentUri, sameDocumentUri } from "./documentUri";

export interface LspSession {
  client: LspClient | null;
  uri: string;
  /** Diagnostics for this document, newest publish wins. */
  diagnostics: LspDiagnostic[];
  /** True once the session has an open document to ask questions about. */
  ready: boolean;
}

</storycode:copy>
<a id="f-5ad96644c888-SYNC_DEBOUNCE_MS"></a>
### `SYNC_DEBOUNCE_MS`

Milliseconds of typing quiet before the server is told the text changed.

<storycode:copy id="f-5ad96644c888-SYNC_DEBOUNCE_MS">
/** Milliseconds of typing quiet before the server is told the text changed. */
const SYNC_DEBOUNCE_MS = 300;

</storycode:copy>
<a id="f-5ad96644c888-useLsp"></a>
### `useLsp`

<storycode:copy id="f-5ad96644c888-useLsp">
export function useLsp(realtime: Realtime, docPath: string, text: string, projectContext = "new"): LspSession {
  const client = useMemo(() => {
    const channel = realtime.lsp();
    return channel ? new LspClient(channel) : null;
  }, [realtime]);
  // This session owns its client: the document's socket goes with the
  // document, and so does everything asked over it.
  useEffect(() => () => client?.dispose(), [client]);
  return useLspOver(client, docPath, text, projectContext);
}

</storycode:copy>
<a id="f-5ad96644c888-workspaceClient"></a>
### `workspaceClient`

The workspace's one client, shared by every plain file.
One, not one per pane, for a reason that is not thrift: every client counts
its request ids from 1, and two clients on ONE channel would each take the
other's replies. Documents avoid this by each having a socket; plain files
have no room and share the workspace socket, so they share the client too.

<storycode:copy id="f-5ad96644c888-workspaceClient">
// The workspace's one client, shared by every plain file.
//
// One, not one per pane, for a reason that is not thrift: every client counts
// its request ids from 1, and two clients on ONE channel would each take the
// other's replies. Documents avoid this by each having a socket; plain files
// have no room and share the workspace socket, so they share the client too.
let workspaceClient: LspClient | null = null;
</storycode:copy>
<a id="f-5ad96644c888-workspaceLspClient"></a>
### `workspaceLspClient`

<storycode:copy id="f-5ad96644c888-workspaceLspClient">
function workspaceLspClient(): LspClient | null {
  if (workspaceClient) return workspaceClient;
  const channel = getWorkspaceRealtime()?.lsp() ?? null;
  if (!channel) return null;
  workspaceClient = new LspClient(channel);
  return workspaceClient;
}

</storycode:copy>
<a id="f-5ad96644c888-resetWorkspaceLsp"></a>
### `resetWorkspaceLsp`

Test seam: forget the workspace client and the connection under it.

<storycode:copy id="f-5ad96644c888-resetWorkspaceLsp">
/** Test seam: forget the workspace client and the connection under it. */
export function resetWorkspaceLsp(): void {
  workspaceClient?.dispose();
  workspaceClient = null;
  resetWorkspaceRealtime();
}

</storycode:copy>
<a id="f-5ad96644c888-useWorkspaceLsp"></a>
### `useWorkspaceLsp`

A language session for a plain file — `src/main.rs`, `app.py` — over the
workspace connection. The same questions, the same answers, at the file's
own path; the server treats a file that is not a document as its own
virtual file.

<storycode:copy id="f-5ad96644c888-useWorkspaceLsp">
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

</storycode:copy>
<a id="f-5ad96644c888-useLspOver"></a>
### `useLspOver`

<storycode:copy id="f-5ad96644c888-useLspOver">
function useLspOver(client: LspClient | null, docPath: string, text: string, projectContext = "new"): LspSession {
  const uri = useMemo(() => documentUri(docPath, projectContext), [docPath, projectContext]);
  const [diagnostics, setDiagnostics] = useState<LspDiagnostic[]>([]);
  const [ready, setReady] = useState(false);
  const versionRef = useRef(1);
  const textRef = useRef(text);
  const lastSent = useRef<string | null>(null);
  const flushRef = useRef<(() => void) | null>(null);
  textRef.current = text;

  useEffect(() => {
    // No document path yet (the view is still loading): nothing to open.
    if (!client || !docPath) return;
    versionRef.current = 1;
    client.didOpen(uri, textRef.current, 1);
    lastSent.current = textRef.current;
    const flush = () => {
      const latest = textRef.current;
      if (latest !== lastSent.current) {
        client.didChange(uri, latest, ++versionRef.current);
        lastSent.current = latest;
      }
    };
    flushRef.current = flush;
    const unregister = client.registerDocumentSync(uri, flush);
    setReady(true);
    const off = client.onDiagnostics((params) => {
      if (sameDocumentUri(params.uri, uri)) setDiagnostics(params.diagnostics);
    });
    return () => {
      off();
      unregister();
      if (flushRef.current === flush) flushRef.current = null;
      client.didClose(uri);
      setReady(false);
    };
  }, [client, uri, docPath]);

  // Debounced full-text sync. Sending on every keystroke would make the bridge
  // re-weave the document per character.
  useEffect(() => {
    if (!client || !ready) return;
    const timer = setTimeout(() => {
      flushRef.current?.();
    }, SYNC_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [client, ready, uri, text]);

  return { client, uri, diagnostics, ready };
}
</storycode:copy>

### Assemble `apps/web/src/lsp/useLsp.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/useLsp.ts"><storycode:paste select="#f-5ad96644c888-LspSession"/><storycode:paste select="#f-5ad96644c888-SYNC_DEBOUNCE_MS"/><storycode:paste select="#f-5ad96644c888-useLsp"/><storycode:paste select="#f-5ad96644c888-workspaceClient"/><storycode:paste select="#f-5ad96644c888-workspaceLspClient"/><storycode:paste select="#f-5ad96644c888-resetWorkspaceLsp"/><storycode:paste select="#f-5ad96644c888-useWorkspaceLsp"/><storycode:paste select="#f-5ad96644c888-useLspOver"/></storycode:file>

## Test the behavior: `drive_one_plain_file`

This is the maintained test fragment from `crates/hickory-cli/tests/lsp_languages.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-7c4650d33fc8-drive_one_plain_file"></a>
### `drive_one_plain_file`

A plain file — no document, no `hick:file` block — gets the same answers,
at its own path, with the project as the root.
Protects docs/guarantees/editor-intelligence/a-plain-file-has-the-same-language-server.md

<storycode:copy id="f-7c4650d33fc8-drive_one_plain_file">
/// A plain file — no document, no `hick:file` block — gets the same answers,
/// at its own path, with the project as the root.
///
/// Protects docs/guarantees/editor-intelligence/a-plain-file-has-the-same-language-server.md
fn drive_one_plain_file(language: &Language) -> bool {
    let dir = tempfile::tempdir().expect("a temp project");
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    borrow_this_repos_servers(dir.path());
    for (path, contents) in language.scaffold {
        let full = dir.path().join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(full, contents).unwrap();
    }
    let file = dir.path().join(language.file);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&file, language.code).unwrap();

    if !server_available(language.id, dir.path()) {
        skip(
            language.id,
            "no language server for it is installed on this machine",
        );
        return false;
    }
    let Some(mut client) = StdioClient::start(dir.path()) else {
        skip(
            language.id,
            "hick-lsp has not been built into this target dir",
        );
        return false;
    };
    let root_uri = format!("file://{}", dir.path().display());
    let uri = format!("file://{}", file.display());
    let id = client.request(
        "initialize",
        json!({ "processId": std::process::id(), "rootUri": root_uri, "capabilities": {} }),
    );
    client
        .wait_for(id, Duration::from_secs(20))
        .expect("the server answers initialize");
    client.notify("initialized", json!({}));
    client.notify(
        "textDocument/didOpen",
        json!({"textDocument": {"uri": uri, "languageId": language.id, "version": 1, "text": language.code}}),
    );

    let budget = Duration::from_secs(60);
    let (line, character) = definition_site(language, 0);
    let hover = client.request_until(
        "textDocument/hover",
        json!({"textDocument": {"uri": uri}, "position": {"line": line, "character": character}}),
        budget,
    );
    let hover_text = hover.map(|v| v.to_string()).unwrap_or_default();
    assert!(
        hover_text.contains(language.symbol),
        "{}: hover in a plain file did not mention {}: {hover_text}",
        language.id,
        language.symbol
    );

    let use_site = language
        .code
        .lines()
        .enumerate()
        .filter(|(_, text)| text.contains(language.symbol))
        .nth(1)
        .map(|(offset, text)| (offset as u32, text.find(language.symbol).unwrap() as u32));
    if let Some((use_line, use_column)) = use_site {
        let found = client.request_until(
            "textDocument/definition",
            json!({"textDocument": {"uri": uri}, "position": {"line": use_line, "character": use_column}}),
            budget,
        );
        let (target_uri, target_line) = found
            .as_ref()
            .and_then(first_location)
            .expect("definition in a plain file is answered");
        assert!(
            target_uri.ends_with(language.file),
            "{}: definition pointed away from the file itself: {target_uri}",
            language.id
        );
        assert_eq!(
            target_line, line,
            "{}: definition landed on the wrong line",
            language.id
        );
    }
    eprintln!(
        "OK {}: hover and definition in a plain file, at its own path",
        language.id
    );
    true
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/components/PlainFilePane.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-2f891a49445f)
- [apps/web/src/lib/fileProblems.ts](contract.md#output-50cf197029a3)
- [apps/web/src/lib/revealLine.ts](../../../mechanisms/apps/web/src/lib/revealLine.ts.md#output-bca672428db0)
- [apps/web/src/lsp/useLsp.ts](contract.md#output-5ad96644c888)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [crates/hick-lsp/src/backend.rs](../../../mechanisms/crates/hick-lsp/src/backend.rs.md#output-c936496a03c1)
- [crates/hick-lsp/src/position_map.rs](../../../mechanisms/crates/hick-lsp/src/position_map.rs.md#output-c77393891cd2)

### Tests and fixtures

- [apps/web/src/components/PlainFilePane.test.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-d923b2fdae48)
- [crates/hickory-cli/tests/lsp_languages.rs](../../../mechanisms/crates/hickory-cli/tests/lsp_languages.rs.md#output-7c4650d33fc8)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
