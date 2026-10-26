<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/fixture.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-687696a9cdc9"></a>
## `apps/web/src/embed/fixture.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-687696a9cdc9-LocalFixture"></a>
### `LocalFixture`

<storycode:copy id="f-687696a9cdc9-LocalFixture">
import { NativeCoachFixture } from "./NativeCoachFixture";
import { StaticUiSurface } from "./StaticUiSurface";
import { GraphFixture } from "./GraphFixture";
import { PictureFixture } from "./PictureFixture";
import { ProfileFixture } from "./ProfileFixture";
import { ReviewFixture } from "./ReviewFixture";
// An external host: imports only the supported embedding entry, no App/site entry.
import { createRoot } from "react-dom/client";
import { useEffect, useState } from "react";
import { BrowserWorkspace, openLocalStorage, openS3Storage, openSyncedStorage } from "./index";
import type { WorkspaceStorage } from "./index";
import { EmbeddingFixture } from "./EmbeddingFixture";


function LocalFixture() {
  const [storage, setStorage] = useState<WorkspaceStorage | null>(null);
  const [error, setError] = useState("");
  useEffect(() => {
    let disposed = false;
    const resources: WorkspaceStorage[] = [];
    void (async () => {
      const params = new URLSearchParams(location.search);
      const local = await openLocalStorage(params.has("s3") ? `fixture-s3-${params.get("local") ?? "one"}` : "fixture"); resources.push(local);
      let current: WorkspaceStorage = local;
      if (params.has("s3")) {
        // Test host callback, intercepted by acceptance tests; no signing or credentials.
        const remote = await openS3Storage({ prefix: "fixture", request: ({ method, key, body, headers, signal }) =>
          fetch(`http://localhost:4178/bucket/${key}`, { method, headers, body: body?.slice().buffer, signal }) });
        resources.push(remote);
        current = await openSyncedStorage(local, remote, "fixture"); resources.push(current);
      }
      if (disposed) for (const resource of resources) resource.close(); else setStorage(current);
    })().catch((e) => { for (const resource of resources) resource.close(); if (!disposed) setError(String(e)); });
    return () => { disposed = true; for (const resource of resources) resource.close(); };
  }, []);
  return storage ? <BrowserWorkspace storage={storage} /> : <p>{error || "Loading local storage…"}</p>;
}
</storycode:copy>
<a id="f-687696a9cdc9-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-687696a9cdc9-expression_statement">
createRoot(document.getElementById("root")!).render(<StaticUiSurface>{location.search.includes("native-coach") ? <NativeCoachFixture /> : location.search.includes("graph") ? <GraphFixture /> : location.search.includes("picture") ? <PictureFixture /> : location.search.includes("profile") ? <ProfileFixture /> : location.search.includes("review") ? <ReviewFixture /> : location.search.includes("workspace") ? <LocalFixture /> : <EmbeddingFixture />}</StaticUiSurface>);
</storycode:copy>

### Assemble `apps/web/src/embed/fixture.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/fixture.tsx"><storycode:paste select="#f-687696a9cdc9-LocalFixture"/><storycode:paste select="#f-687696a9cdc9-expression_statement"/></storycode:file>

</storycode:doc>
