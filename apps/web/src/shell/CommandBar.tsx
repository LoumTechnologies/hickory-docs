// The search box across the top of the window.
//
// It is one field, and it is deliberately not four. A window that grows a
// separate control for "open a file", "find a symbol", "search the folder",
// and "run a command" has four things to learn and four places to look; one
// field that understands a prefix has one, and the prefixes are the same ones
// every editor already taught people.
//
//   (nothing)   — search open files and folder contents, ranked
//   `>`         — a command
//   `?`         — search open files and folder contents, ranked (an explicit spelling
//                 for people who learned the earlier prefix)
//   `:`         — a line number in the file that is open
//
// The bar itself is dumb: it parses the prefix, asks whoever mounted it for
// candidates, and reports the pick. What any of those things MEAN belongs to
// the workspace, which is the only thing that knows what is open.

import { isAction } from "../lib/keymap";
import "./commandPalette.css";
import { useEffect, useId, useLayoutEffect, useMemo, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiField } from "../lib/uiActions";

const OPEN_COMMAND_BAR = "hickory-open-command-bar";
export function openCommandBar(commands = false) {
  window.dispatchEvent(new CustomEvent(OPEN_COMMAND_BAR, { detail: commands }));
}

/** What a query is asking for. */
export type CommandMode = "files" | "command" | "content" | "line";

export interface CommandItem {
  id: string;
  label: string;
  /** The dim second line — a path, a match, a keyboard shortcut. */
  detail?: string;
  run: () => void;
  /** Command-mode callbacks need explicit policy review; navigation is separate. */
  agentAllowed?: boolean;
}

/** The mode a raw query is in, and the query with its prefix removed. */
export function parseQuery(raw: string): { mode: CommandMode; term: string } {
  const trimmed = raw.trimStart();
  if (trimmed.startsWith(">")) return { mode: "command", term: trimmed.slice(1).trim() };
  if (trimmed.startsWith("?")) return { mode: "content", term: trimmed.slice(1).trim() };
  if (trimmed.startsWith(":")) return { mode: "line", term: trimmed.slice(1).trim() };
  return { mode: "content", term: trimmed.trim() };
}

/** What the bar says it will do, so the prefixes are discoverable without a
 * help page. */
export function modeHint(mode: CommandMode): string {
  switch (mode) {
    case "command":
      return "Commands";
    case "content":
      return "Search open files and folder";
    case "line":
      return "Go to line";
    case "files":
      return "Files";
  }
}

export function CommandBar({
  candidates,
  placeholder = "Search",
  overlay = false,
}: {
  /** Answers for a query. Async because searching contents is a request. */
  candidates: (mode: CommandMode, term: string) => Promise<CommandItem[]> | CommandItem[];
  placeholder?: string;
  overlay?: boolean;
}) {
  const [raw, setRaw] = useState("");
  const [open, setOpen] = useState(false);
  const [results, setResults] = useState<{ raw: string; items: CommandItem[] } | null>(null);
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listId = useId();
  const items = results?.raw === raw ? results.items : [];
  const loading = results?.raw !== raw;
  const previousFocus = useRef<HTMLElement | null>(null);
  const openRef = useRef(open); openRef.current = open;
  const { mode, term } = useMemo(() => parseQuery(raw), [raw]);
  const reveal = (commands: boolean) => {
      if (!openRef.current) previousFocus.current = document.activeElement as HTMLElement | null;
      setRaw(commands ? ">" : "");
      setOpen(true);
      inputRef.current?.focus();
      inputRef.current?.select();
  };
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (inputRef.current?.closest("[hidden]:not(.command-palette)")) return;
      const commands = isAction(event, "search.commands");
      if (!commands && !isAction(event, "search.quickOpen")) return;
      event.preventDefault();
      reveal(commands);
    };
    const onOpen = (event: Event) => reveal((event as CustomEvent<boolean>).detail === true);
    window.addEventListener("keydown", onKey);
    window.addEventListener(OPEN_COMMAND_BAR, onOpen);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener(OPEN_COMMAND_BAR, onOpen);
    };
  }, []);
  useLayoutEffect(() => {
    if (overlay && open) { inputRef.current?.focus(); inputRef.current?.select(); }
  }, [overlay, open]);
  const dismiss = () => {
    setOpen(false);
    setRaw("");
    if (overlay && previousFocus.current?.isConnected) previousFocus.current.focus();
    else inputRef.current?.blur();
  };

  useEffect(() => {
    if (!open) return;
    setResults(null);
    setSelected(0);
    let live = true;
    const timer = window.setTimeout(() => {
      void Promise.resolve(candidates(mode, term)).then(
        (found) => {
          if (!live) return;
          setResults({ raw, items: found });
          setSelected(0);
        },
        () => live && setResults({ raw, items: [] }),
      );
      // A keystroke is not a question; a pause is. Short, because this is the
      // control people type into fastest.
    }, 120);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
  }, [open, raw, mode, term, candidates]);

  useEffect(() => {
    if (open) document.getElementById(`${listId}-${selected}`)?.scrollIntoView?.({ block: "nearest" });
  }, [open, selected, results, listId]);

  const choose = (item: CommandItem | undefined) => {
    if (!item) return;
    dismiss();
    return item.run();
  };
  const parent = `command-search:${listId}`;
  const revision = useUiRevision(() => [raw, open, selected, results?.raw, items.map(item => [item.id, item.label, item.detail, item.agentAllowed])]);
  const commands: { key: string; command: UiCommand }[] = [];
  function add(id: string, label: string, parameters: Record<string, UiField>, execute: UiCommand["execute"], enabled = true, group = parent) {
    commands.push({ key: `${parent}/${id}`, command: { label, parent: group, subject: parent, parameters, prerequisites: ["Current query and returned candidate identity; command-mode policy applies"], read: () => ({ revision: revision(), enabled, reason: enabled ? undefined : "Wait for the current search response" }), execute } });
  }
  add("visibility", "Set command search visibility", { open: { type: "boolean" }, commands: { type: "boolean" } }, async args => { if (args.open) reveal(Boolean(args.commands)); else dismiss(); return { status: "completed", value: { open: Boolean(args.open) } }; }, true, "command-search-disclosures");
  if (open) {
    add("input", "Set command search query", { query: { type: "string", maxLength: 8192 } }, async args => { setRaw(String(args.query)); return { status: "completed", value: { dispatch: "query requested", result: "Observe the current candidates before choosing" } }; });
    add("read", "Read command search bytes", { offset: { type: "number", minimum: 0, maximum: 100000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ query: raw, mode, loading, selected, items: items.map(item => ({ id: item.id, label: item.label, detail: item.detail, agent_allowed: mode !== "command" || item.agentAllowed === true })) }));
      const offset = Number(args.offset), end = Math.min(bytes.length, offset + Number(args.limit));
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset exceeds current command search" };
      return { status: "completed", value: { encoding: "JSON encoded as UTF-8 bytes", bytes: [...bytes.slice(offset, end)], span: [offset, end], total_bytes: bytes.length, next: end < bytes.length ? end : null } };
    });
    add("select", "Select command search candidate", { index: { type: "number", minimum: 0, maximum: 1000000, integer: true } }, async args => { if (!items[Number(args.index)]) return { status: "unavailable", reason: "Index does not belong to this query's results" }; setSelected(Number(args.index)); return { status: "completed", value: { selected: args.index } }; }, !loading);
    add("choose", "Choose command search candidate", { id: { type: "string", maxLength: 8192 } }, async (args, actor) => {
      const item = items.find(item => item.id === args.id);
      if (!item) return { status: "unavailable", reason: "Candidate does not belong to this query's current result" };
      if (actor.kind === "agent" && mode === "command" && item.agentAllowed !== true) return { status: "unavailable", reason: "This palette callback has no agent policy adapter. For document runs use the source-bound Run/Verify saved document controls; other command adapters remain integration work" };
      await choose(item); return { status: "completed", value: { dispatch: "requested", item: item.id, result: "Observe the actual route, dialog or source-bound job receipt" } };
    }, !loading);
  }
  useUiCommands(commands);

  return (
    <div className={overlay ? "command-palette" : "command-bar"} hidden={overlay && !open} role="search"
      onMouseDown={event => { if (overlay && event.target === event.currentTarget) dismiss(); }}>
      <div className={overlay ? "command-bar command-bar--overlay" : undefined}>
      <input
        ref={inputRef}
        className="command-bar__input"
        type="text"
        value={raw}
        placeholder={placeholder}
        aria-label={modeHint(mode)}
        aria-expanded={open}
        aria-controls={open ? listId : undefined}
        aria-activedescendant={items[selected] ? `${listId}-${selected}` : undefined}
        onFocus={() => setOpen(true)}
        // Result buttons keep the input focused on mousedown. An overlay
        // closes immediately when focus leaves it, so reopening cannot be
        // cancelled by a timeout from its previous dismissal.
        onBlur={(event) => {
          if (overlay) {
            if (!event.currentTarget.parentElement?.contains(event.relatedTarget as Node | null)) setOpen(false);
          } else window.setTimeout(() => setOpen(false), 120);
        }}
        onChange={(event) => {
          setRaw(event.target.value);
          setOpen(true);
        }}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.preventDefault();
            dismiss();
          } else if (event.key === "ArrowDown") {
            event.preventDefault();
            setSelected((i) => Math.max(0, Math.min(i + 1, items.length - 1)));
          } else if (event.key === "ArrowUp") {
            event.preventDefault();
            setSelected((i) => Math.max(i - 1, 0));
          } else if (event.key === "Enter") {
            event.preventDefault();
            choose(items[selected]);
          }
        }}
      />
      {open && (
        <div className="command-bar__panel">
          <p className="command-bar__hint">{modeHint(mode)}</p>
          {items.length === 0 ? (
            <p className="command-bar__empty muted">
              {loading ? "Searching…" : term ? "Nothing matches." : "Start typing."}
            </p>
          ) : (
            <ul className="command-bar__list" role="listbox" id={listId} aria-label={modeHint(mode)}>
              {items.map((item, index) => (
                <li key={item.id}>
                  <button
                    type="button"
                    role="option"
                    id={`${listId}-${index}`}
                    aria-selected={index === selected}
                    className={`command-bar__item${index === selected ? " command-bar__item--on" : ""}`}
                    onMouseEnter={() => setSelected(index)}
                    onMouseDown={(event) => event.preventDefault()}
                    onClick={() => choose(item)}
                  >
                    <span className="command-bar__label">{item.label}</span>
                    {item.detail && (
                      <span className="command-bar__detail mono">{item.detail}</span>
                    )}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
      </div>
    </div>
  );
}
