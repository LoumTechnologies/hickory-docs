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
createRoot(document.getElementById("root")!).render(<StaticUiSurface>{location.search.includes("graph") ? <GraphFixture /> : location.search.includes("picture") ? <PictureFixture /> : location.search.includes("profile") ? <ProfileFixture /> : location.search.includes("review") ? <ReviewFixture /> : location.search.includes("workspace") ? <LocalFixture /> : <EmbeddingFixture />}</StaticUiSurface>);
