// Untitled work has a durable identity per workspace/window, even before Save.
import { useEffect, useRef, useState } from "react";
import type { Dispatch, MutableRefObject, SetStateAction } from "react";
import { api } from "../api/client";
import { useDraftKeeper } from "../lib/drafts";
import { UNTITLED_RECOVERY_KEY } from "../lib/newDoc";
import { panes, type Layout } from "../shell/layout";
import { openUntitledTab } from "./workspaceState";

export function useUntitledRecovery({ hydrated, layoutRef, setLayout, sourcesRef, setSources, initialBaselines }: {
  hydrated: boolean;
  layoutRef: MutableRefObject<Layout>;
  setLayout: Dispatch<SetStateAction<Layout>>;
  sourcesRef: MutableRefObject<Record<string, string>>;
  setSources: Dispatch<SetStateAction<Record<string, string>>>;
  initialBaselines: Record<string, string>;
}) {
  const [ready, setReady] = useState(false);
  const [baselines, setBaselines] = useState(initialBaselines);
  const baselinesRef = useRef(baselines);
  baselinesRef.current = baselines;
  useEffect(() => {
    if (!hydrated) return;
    let cancelled = false;
    void api.drafts().then(({ drafts }) => {
      if (cancelled) return;
      const draft = drafts.find((entry) => entry.path === UNTITLED_RECOVERY_KEY);
      if (draft && draft.contents !== draft.base) {
        const next = openUntitledTab(layoutRef.current);
        const tab = panes(next.root).flatMap((pane) => pane.tabs).find((tab) => tab.kind === "untitled")!;
        // Do not replace text typed while hydration was in flight.
        const current = sourcesRef.current[tab.id];
        if (current === undefined || current === (baselinesRef.current[tab.id] ?? "")) {
          setBaselines((previous) => ({ ...previous, [tab.id]: draft.base }));
          setSources((previous) => ({ ...previous, [tab.id]: draft.contents }));
          layoutRef.current = next;
          setLayout(next);
        }
      }
    }).catch(() => {}).finally(() => { if (!cancelled) setReady(true); });
    return () => { cancelled = true; };
  }, [hydrated, layoutRef, setLayout, sourcesRef, setSources]);

  const activeTab = panes(layoutRef.current.root).flatMap((pane) => pane.tabs).find((tab) => tab.kind === "untitled");
  const lastTab = useRef(activeTab);
  if (activeTab) lastTab.current = activeTab;
  const previous = useRef<{ id: string; dirty: boolean } | null>(null);
  useEffect(() => {
    if (!ready || !activeTab) return;
    const base = baselines[activeTab.id] ?? "";
    const dirty = (sourcesRef.current[activeTab.id] ?? base) !== base;
    if (previous.current?.id === activeTab.id && previous.current.dirty && !dirty) {
      void api.discardDraft(UNTITLED_RECOVERY_KEY).catch(() => {});
    }
    previous.current = { id: activeTab.id, dirty };
  });
  const discard = useDraftKeeper({
    path: UNTITLED_RECOVERY_KEY,
    enabled: ready && !!activeTab,
    read: () => {
      // Cleanup can run after the tab has left the layout. Its last buffer
      // still lives in the workspace; absence must never flush empty text.
      const tab = lastTab.current;
      const base = tab ? baselinesRef.current[tab.id] ?? "" : "";
      return { contents: tab ? sourcesRef.current[tab.id] ?? base : base, base };
    },
  });
  useEffect(() => {
    const onSaved = () => discard();
    window.addEventListener("hickory-untitled-saved", onSaved);
    return () => window.removeEventListener("hickory-untitled-saved", onSaved);
  }, [discard]);
  return { ready, baselines };
}
