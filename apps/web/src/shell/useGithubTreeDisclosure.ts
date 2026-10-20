import { useEffect, useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand } from "../lib/uiActions";

/** Lazy provider disclosure: parent reads do not fetch hidden content. */
export function useGithubTreeDisclosure<T>(label: string, source: unknown, fetch: () => Promise<T>, blocked?: string, renameable = true) {
  const parent = `github-tree:${useId()}`;
  const key = JSON.stringify(source);
  const live = useRef(key); live.current = key;
  const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const [open, setOpen] = useState(false);
  const [renaming, setRenaming] = useState(false);
  const [capture, setCapture] = useState<{ key: string; value: T } | null>(null);
  const [failure, setFailure] = useState<{ key: string; reason: string } | null>(null);
  const [job, setJob] = useState<{ id: string; key: string; status: string } | null>(null);
  const pending = useRef<{ id: string; key: string } | null>(null);
  const detail = capture?.value ?? null;
  const changed = !!capture && capture.key !== key;
  const error = failure?.key === key ? failure.reason : null;
  const load = () => {
    if (blocked || pending.current?.key === key) return;
    const current = { id: crypto.randomUUID(), key }; pending.current = current;
    setFailure(null); setJob({ ...current, status: "loading" });
    void fetch().then(value => {
      if (!mounted.current || live.current !== key || pending.current !== current) return;
      setCapture({ key, value }); setJob({ ...current, status: "completed" });
    }, cause => {
      if (!mounted.current || live.current !== key || pending.current !== current) return;
      setFailure({ key, reason: cause instanceof Error ? cause.message : String(cause) });
      setJob({ ...current, status: "failed" });
    }).finally(() => { if (pending.current === current) pending.current = null; });
  };
  const visibility = (value: boolean) => { setOpen(value); if (value && (!detail || changed)) load(); };
  useEffect(() => { if (open && (!detail || changed)) load(); }, [key]);
  const rename = () => { visibility(true); setRenaming(true); };
  const revision = useUiRevision(() => [key, open, renaming, capture, failure, job, blocked]);
  const actions: { id: string; label: string; parameters?: UiCommand["parameters"]; enabled?: () => boolean; execute: UiCommand["execute"] }[] = [
    { id: "visibility", label: `Set ${label} visibility`, parameters: { open: { type: "boolean" } }, execute: async args => {
      visibility(args.open as boolean); return { status: "completed", value: { open: args.open, content_loaded: !!detail, capture_matches_source: !changed, fetch_blocked: blocked ?? null } };
    } },
  ];
  if (open) actions.push(
    { id: "refresh", label: `Refresh ${label}`, enabled: () => !blocked && pending.current?.key !== key,
      execute: async () => { load(); return { status: "completed", value: { refresh_requested: true, loaded: false } }; } },
    { id: "cancel", label: `Discard pending ${label} read`, enabled: () => pending.current?.key === key,
      execute: async () => {
        const current = pending.current; pending.current = null;
        if (current) setJob({ ...current, status: "discarded" });
        return { status: "completed", value: { result_discarded: true, host_request_cancelled: false } };
      } },
  );
  if (open && renameable) actions.push({ id: "rename", label: `Rename ${label}`, execute: async () => {
    rename(); return { status: "completed", value: { rename_editor_open: true } };
  } });
  const invoke = useUiCommands(actions.map(action => ({ key: `${parent}:${action.id}`, command: {
    parent, subject: parent, label: action.label, parameters: action.parameters ?? {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: action.enabled?.() ?? true,
      reason: blocked, value: { open, renaming, loaded: !!detail, capture_matches_source: !changed, error, fetch_blocked: blocked ?? null } }), execute: action.execute,
  } })));
  useUiReading(`${label} disclosure status`, { open, renaming, loaded: !!detail, capture_matches_source: !changed, error, blocked: blocked ?? null,
    current_source: source, captured_source: capture ? JSON.parse(capture.key) as unknown : null,
    job: job?.key === key ? { id: job.id, status: job.status } : null });
  const toggle = () => void invoke(`${parent}:visibility`, { open: !open }).catch(() => {});
  return { open, detail, changed, error, renaming, setRenaming, load, toggle, rename };
}
