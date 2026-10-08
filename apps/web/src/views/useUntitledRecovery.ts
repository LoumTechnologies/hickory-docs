// Each unnamed buffer keeps its own recovery draft outside the project.
import { useEffect, useRef, useState } from "react";
import type { Dispatch, MutableRefObject, SetStateAction } from "react";
import { api } from "../api/client";
import { DRAFT_POLL_MS } from "../lib/drafts";
import { UNTITLED_RECOVERY_KEY, untitledDraftKey } from "../lib/newDoc";
import { open, panes, type Layout, type Node } from "../shell/layout";
import { openUntitledTab } from "./workspaceState";

function reidentify(layout: Layout, oldId: string, id: string): Layout {
  const visit = (node: Node): Node => node.type === "pane"
    ? { ...node, tabs: node.tabs.map(tab => tab.id === oldId ? { ...tab, id } : tab) }
    : { ...node, children: node.children.map(visit) };
  return { ...layout, root: visit(layout.root) };
}

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
      let next = layoutRef.current;
      const identities = new Map<string, string>();
      const sources: Record<string, string> = {};
      const bases: Record<string, string> = {};
      for (const draft of drafts.filter(draft => draft.path.startsWith("untitled:") && draft.contents !== draft.base)) {
        const id = draft.path.slice("untitled:".length);
        let tab = panes(next.root).flatMap(pane => pane.tabs).find(tab => tab.kind === "untitled" && tab.id === id);
        // Older versions kept only one unnamed draft. Adopt that into the
        // startup buffer without overwriting anything typed during hydration.
        if (draft.path === UNTITLED_RECOVERY_KEY) tab = panes(next.root).flatMap(pane => pane.tabs).find(tab => tab.kind === "untitled");
        // The startup introduction is a fallback, not another recovered
        // document. Reuse it for the first draft even when that draft began
        // blank or the bundled introduction changed since the previous run.
        if (!tab && Object.keys(sources).length === 0) {
          const startup = panes(next.root).flatMap(pane => pane.tabs).find(tab => tab.kind === "untitled" && tab.id in baselinesRef.current && (sourcesRef.current[tab.id] ?? baselinesRef.current[tab.id]) === baselinesRef.current[tab.id]);
          if (startup) {
            identities.set(startup.id, id);
            next = reidentify(next, startup.id, id);
            tab = { ...startup, id };
          }
        }
        if (!tab) {
          next = openUntitledTab(next, true);
          const pane = panes(next.root).find(pane => pane.id === next.focus)!;
          tab = pane.tabs[pane.active];
          if (draft.path !== UNTITLED_RECOVERY_KEY) {
            identities.set(tab.id, id);
            next = reidentify(next, tab.id, id);
            tab = { ...tab, id };
          }
        }
        const current = sourcesRef.current[tab.id];
        if (current !== undefined && current !== (baselinesRef.current[tab.id] ?? "")) continue;
        sources[tab.id] = draft.contents;
        bases[tab.id] = draft.base;
        if (draft.path === UNTITLED_RECOVERY_KEY) {
          void api.saveDraft({ ...draft, path: untitledDraftKey(tab.id) }).then(() => api.discardDraft(UNTITLED_RECOVERY_KEY)).catch(() => {});
        }
      }
      if (Object.keys(sources).length === 0) return;
      setBaselines(previous => ({ ...previous, ...bases }));
      setSources(previous => ({ ...previous, ...sources }));
      const recovered = panes(next.root).flatMap(pane => pane.tabs).filter(tab => tab.id in sources);
      setLayout(current => {
        // Folder hydration can have added Files while drafts were loading.
        // Apply recovered buffers to that live arrangement, never replace it
        // with the earlier snapshot and lose newly opened panes.
        let merged = current;
        for (const [oldId, id] of identities) merged = reidentify(merged, oldId, id);
        for (const tab of recovered) {
          if (panes(merged.root).some(pane => pane.tabs.some(entry => entry.id === tab.id))) continue;
          const editor = panes(merged.root).find(pane => !pane.tabs.some(entry => entry.kind === "tree"));
          merged = open(merged, tab, editor?.id);
        }
        layoutRef.current = merged;
        return merged;
      });
    }).catch(() => {}).finally(() => { if (!cancelled) setReady(true); });
    return () => { cancelled = true; };
  }, [hydrated, layoutRef, setLayout, sourcesRef, setSources]);

  useEffect(() => {
    if (!ready) return;
    const written = new Map<string, string>();
    const saved = new Set<string>();
    const pending = new Map<string, Promise<unknown>>();
    const onSaved = (event: Event) => {
      const id = (event as CustomEvent<string>).detail;
      saved.add(id);
      void api.discardDraft(untitledDraftKey(id)).catch(() => {});
    };
    const onDiscarded = (event: Event) => {
      const { id, waitUntil } = (event as CustomEvent<{ id: string; waitUntil: (operation: Promise<unknown>) => void }>).detail;
      saved.add(id);
      waitUntil((pending.get(id) ?? Promise.resolve()).catch(() => {}).then(() => api.discardDraft(untitledDraftKey(id))));
    };
    const flush = () => {
      for (const tab of panes(layoutRef.current.root).flatMap(pane => pane.tabs).filter(tab => tab.kind === "untitled")) {
        if (saved.has(tab.id)) continue;
        const base = baselinesRef.current[tab.id] ?? "";
        const contents = sourcesRef.current[tab.id] ?? base;
        if (contents === base) {
          if (written.has(tab.id)) {
            written.delete(tab.id);
            void api.discardDraft(untitledDraftKey(tab.id)).catch(() => {});
          }
        } else if (written.get(tab.id) !== contents) {
          written.set(tab.id, contents);
          const write = (pending.get(tab.id) ?? Promise.resolve()).then(() =>
            api.saveDraft({ path: untitledDraftKey(tab.id), contents, base, saved_at: Date.now() })
          ).catch(() => written.delete(tab.id));
          pending.set(tab.id, write);
        }
      }
    };
    const timer = window.setInterval(flush, DRAFT_POLL_MS);
    window.addEventListener("pagehide", flush);
    window.addEventListener("hickory-untitled-saved", onSaved);
    window.addEventListener("hickory-untitled-discarded", onDiscarded);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("pagehide", flush);
      window.removeEventListener("hickory-untitled-saved", onSaved);
      window.removeEventListener("hickory-untitled-discarded", onDiscarded);
      flush();
    };
  }, [ready, layoutRef, sourcesRef]);
  // Clear a persisted draft immediately when undo returns its buffer to base.
  useEffect(() => {
    if (!ready) return;
    for (const [id, contents] of Object.entries(sourcesRef.current)) {
      if (contents === (baselinesRef.current[id] ?? "")) void api.discardDraft(untitledDraftKey(id)).catch(() => {});
    }
  });
  return { ready, baselines };
}
