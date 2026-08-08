// A fresh live-room connection per generated output file, owned by whichever
// view currently has that file open (SplitView / OutputView) — the same
// construct-and-close-on-change shape DocumentView uses for the Document
// room's `realtime`, just keyed on `(docId, path)` instead of `docId` alone
// since the file being viewed can change without the doc itself changing.

import { useEffect, useMemo } from "react";
import { WsRealtime, getSharedRealtime, type Realtime } from "./realtime";

/** `null` while there is no active file yet (nothing to connect to). */
export function useOutputRealtime(docId: string, path: string | null): Realtime | null {
  const realtime = useMemo(() => {
    if (!path) return null;
    // A shared realtime is registered only in mock mode (main.tsx) or by a
    // test standing in for it — reuse it like DocumentView does, rather
    // than opening a socket that has no server behind it.
    const shared = getSharedRealtime();
    if (shared) return shared;
    return new WsRealtime(`output:${docId}:${path}`);
  }, [docId, path]);

  useEffect(() => {
    return () => {
      if (realtime && realtime !== getSharedRealtime()) realtime.close();
    };
  }, [realtime]);

  return realtime;
}
