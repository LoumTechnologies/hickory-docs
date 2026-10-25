<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/DebugRail.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Captured reading surfaces share owned native actions](../../../../../contract.md)
- [Debug controls preserve editor space](../../../../../../../debugging/debug-controls-preserve-editor-space/contract.md)

Captured SHA-256: `224cbdfb8dde2e348728beb9295da89239ebd1a7bda781fb4c5b7130481b3ae6`.

<book:entry path="apps/web/src/debug/DebugRail.tsx">
import { useDebugLaunchActions } from "./useDebugLaunchActions";
import { bindingOf, formatBinding, isAction } from "../lib/keymap";
import { useEffect, useRef, type ReactNode } from "react";
import { createPortal } from "react-dom";
import type { EditorView } from "@codemirror/view";
import { fileBlocksOf, languageForBlock } from "../editor/hickDoc";
import { structureOf } from "../editor/wysiwyg";
import { RAIL_SYNC_EVENT } from "../editor/lineHighlight";
import { isDebuggable } from "./languages";

/** Pure placement: active controls stay reachable when their line scrolls away. */
export function debugRailTop(wanted: number, height: number, available: number, sticky: boolean): number | null {
  if (sticky) return Math.max(0, Math.min(wanted, Math.max(0, available - height)));
  return wanted + height < 0 || wanted > available ? null : wanted;
}

export function DebugRailGroup({ view, at, sticky = false, children }: {
  view: EditorView; at: number; sticky?: boolean; children: ReactNode;
}) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const element = ref.current, rail = element?.parentElement;
    if (!element || !rail) return;
    let frame: number | null = null, disposed = false;
    const schedule = () => {
      if (disposed || frame !== null) return;
      frame = requestAnimationFrame(() => {
        frame = null;
        if (disposed || !view.dom.isConnected) return;
        view.requestMeasure({
          read: () => {
            const pos = Math.max(0, Math.min(at, view.state.doc.length));
            const wanted = view.lineBlockAt(pos).top + view.documentTop - rail.getBoundingClientRect().top;
            const top = debugRailTop(wanted, element.offsetHeight, rail.clientHeight, sticky);
            const menu = element.querySelector<HTMLElement>("details[open] .more-actions__items, .debug-strip > .debug-strip__error, .debug-strip__missing");
            const menuTop = Math.max(-(top ?? 0), Math.min(0, rail.clientHeight - (menu?.offsetHeight ?? 0) - (top ?? 0)));
            return { top, menuTop, available: rail.clientHeight };
          },
          write: ({ top, menuTop, available }) => {
            if (disposed) return;
            element.style.visibility = top === null ? "hidden" : "visible";
            element.style.top = `${top ?? 0}px`;
            element.style.setProperty("--debug-menu-top", `${menuTop}px`);
            element.style.setProperty("--debug-available", `${available}px`);
          },
        });
      });
    };
    schedule();
    element.addEventListener("toggle", schedule, true);
    view.scrollDOM.addEventListener("scroll", schedule, { passive: true });
    window.addEventListener("resize", schedule);
    window.addEventListener(RAIL_SYNC_EVENT, schedule);
    const observer = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(schedule);
    observer?.observe(view.scrollDOM); observer?.observe(view.contentDOM); observer?.observe(element);
    const timer = window.setInterval(schedule, 500);
    return () => {
      disposed = true;
      if (frame !== null) cancelAnimationFrame(frame);
      clearInterval(timer); observer?.disconnect();
      element.removeEventListener("toggle", schedule, true);
      view.scrollDOM.removeEventListener("scroll", schedule);
      window.removeEventListener("resize", schedule);
      window.removeEventListener(RAIL_SYNC_EVENT, schedule);
    };
  }, [view, at, sticky]);
  return <div ref={ref} className={`cm-debug-rail__group${sticky ? " is-active" : ""}`} data-source-offset={at}>{children}</div>;
}

/** A single right-hand action column shared by launch icons and live controls. */
export function DebugLaunchRail({ view, onStart, onProfiles, plainPath, enabled = true }: {
  view: EditorView | null; onStart?: (path: string, at?: number) => void; onProfiles?: (path: string, at?: number) => void; plainPath?: string; enabled?: boolean;
}) {
  const targets: { path: string; at: number; end: number }[] = [];
  if (view && onStart && enabled) {
    if (plainPath) targets.push({ path: plainPath, at: 0, end: view.state.doc.length });
    else {
      const structure = structureOf(view.state);
      for (const block of fileBlocksOf(structure)) {
        if (block.attrs.path && isDebuggable(block.attrs.language ?? languageForBlock(structure, block))) {
          targets.push({ path: block.attrs.path, at: block.from, end: block.to });
        }
      }
    }
  }
  const controls = useDebugLaunchActions(view, targets.flatMap(target => [
    { key: `start:${target.path}:${target.at}`, label: `Debug ${target.path}`, run: () => onStart?.(target.path, plainPath ? undefined : target.at) },
    ...(onProfiles ? [{ key: `profiles:${target.path}:${target.at}`, label: `Launch profiles for ${target.path}`, run: () => onProfiles(target.path, plainPath ? undefined : target.at) }] : []),
  ]));
  const start = (target: typeof targets[number], profiles = false) => { void controls.launch(`${profiles ? "profiles" : "start"}:${target.path}:${target.at}`); };
  const current = useRef({ targets, start }); current.current = { targets, start };
  useEffect(() => {
    if (!view) return;
    const key = (event: KeyboardEvent) => {
      if (event.defaultPrevented || !debugViewFocused(view) || !(isAction(event, "debug.continue") || isAction(event, "debug.start"))) return;
      if (debugRailElement(view)?.querySelector(".is-active")) return;
      const {targets,start} = current.current, pos = view.state.selection.main.head;
      const target = targets.find(t => t.at <= pos && pos <= t.end) ?? (targets.length === 1 ? targets[0] : null);
      if (target) { event.preventDefault(); start(target); }
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [view, plainPath]);
  return <div className="cm-debug-rail" aria-label="Script debugging">
    {controls.error && <p role="alert">{controls.error}</p>}
    {view && targets.map(target => <DebugRailGroup key={`${target.path}:${target.at}`} view={view} at={target.at}>
      <button type="button" className="cm-debug-start debug-icon" aria-label={`Debug ${target.path}`}
        data-tip={`Debug ${target.path} · installs the debugger if needed`} onClick={() => start(target)}>▷</button>
      {onProfiles && <button type="button" className="debug-icon" aria-label={`Launch profiles for ${target.path}`} data-tip="Choose a Run or Debug profile" onClick={()=>start(target, true)}>⋯</button>}
    </DebugRailGroup>)}
  </div>;
}

export function DebugGutterControls({ view, line, program, children }: {
  view: EditorView | null; line?: number | null; program?: string | null; children: ReactNode;
}) {
  const rail = view ? debugRailElement(view) : null;
  if (!view || !rail) return null;
  let at = line === null || line === undefined ? 0 : view.state.doc.line(Math.max(1, Math.min(line + 1, view.state.doc.lines))).from;
  if (line === null || line === undefined) {
    const structure = structureOf(view.state);
    at = fileBlocksOf(structure).find(block => block.attrs.path === program)?.from ?? 0;
  }
  return createPortal(<DebugRailGroup view={view} at={at} sticky>{children}</DebugRailGroup>, rail);
}


export function debugRailElement(view: EditorView): HTMLElement | null {
  const host = view.dom.closest(".editor-cm-host");
  return [...(host?.parentElement?.querySelectorAll<HTMLElement>(".cm-debug-rail") ?? [])].find(rail => !host?.contains(rail)) ?? null;
}
export function debugViewFocused(view: EditorView): boolean {
  const active = document.activeElement;
  return active?.closest(".cm-editor") === view.dom || !!(active && debugRailElement(view)?.contains(active));
}

/** Projection editors use the same launch action as ordinary scripts. */
export function DebugLaunchAction({ view, label, onStart, disabled = false, hint, onProfiles }: {
  view: EditorView | null; label: string; onStart: () => void; disabled?: boolean; hint?: string; onProfiles?: () => void;
}) {
  const rail = view ? debugRailElement(view) : null;
  const controls = useDebugLaunchActions(view, view && rail ? [
    { key: "start", label, disabled, run: onStart },
    ...(onProfiles ? [{ key: "profiles", label: `Launch profiles for ${label}`, disabled, run: onProfiles }] : []),
  ] : []);
  const start = () => { void controls.launch("start"); };
  const callback = useRef({onStart: start,disabled}); callback.current = {onStart: start,disabled};
  useEffect(() => {
    if (!view) return;
    const key = (event: KeyboardEvent) => {
      if (event.defaultPrevented || !debugViewFocused(view) || callback.current.disabled || !(isAction(event, "debug.continue") || isAction(event, "debug.start"))) return;
      if (debugRailElement(view)?.querySelector(".is-active")) return;
      event.preventDefault(); callback.current.onStart();
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [view]);
  return view && rail ? createPortal(<DebugRailGroup view={view} at={0}>
    {controls.error && <p role="alert">{controls.error}</p>}
    <button type="button" className="cm-debug-start debug-icon" aria-label={label} data-tip={hint ?? `${label} · ${formatBinding(bindingOf("debug.start") || bindingOf("debug.continue"))}`} disabled={disabled} onClick={start}>▷</button>
    {onProfiles && <button type="button" className="debug-icon" aria-label={`Launch profiles for ${label}`} data-tip="Choose a Run or Debug profile" disabled={disabled} onClick={() => void controls.launch("profiles")}>⋯</button>}
  </DebugRailGroup>, rail) : null;
}
</book:entry>

</book:doc>
