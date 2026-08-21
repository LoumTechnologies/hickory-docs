// The search box across the top of the window.
//
// It is one field, and it is deliberately not four. A window that grows a
// separate control for "open a file", "find a symbol", "search the folder",
// and "run a command" has four things to learn and four places to look; one
// field that understands a prefix has one, and the prefixes are the same ones
// every editor already taught people.
//
//   (nothing)   — documents and files in this folder, by name
//   `>`         — a command
//   `?`         — search the folder's CONTENTS, ranked (the same engine ⌘⇧F
//                 has always used)
//   `:`         — a line number in the file that is open
//
// The bar itself is dumb: it parses the prefix, asks whoever mounted it for
// candidates, and reports the pick. What any of those things MEAN belongs to
// the workspace, which is the only thing that knows what is open.

import { useEffect, useMemo, useRef, useState } from "react";

/** What a query is asking for. */
export type CommandMode = "files" | "command" | "content" | "line";

export interface CommandItem {
  id: string;
  label: string;
  /** The dim second line — a path, a match, a keyboard shortcut. */
  detail?: string;
  run: () => void;
}

/** The mode a raw query is in, and the query with its prefix removed. */
export function parseQuery(raw: string): { mode: CommandMode; term: string } {
  const trimmed = raw.trimStart();
  if (trimmed.startsWith(">")) return { mode: "command", term: trimmed.slice(1).trim() };
  if (trimmed.startsWith("?")) return { mode: "content", term: trimmed.slice(1).trim() };
  if (trimmed.startsWith(":")) return { mode: "line", term: trimmed.slice(1).trim() };
  return { mode: "files", term: trimmed.trim() };
}

/** What the bar says it will do, so the prefixes are discoverable without a
 * help page. */
export function modeHint(mode: CommandMode): string {
  switch (mode) {
    case "command":
      return "Commands";
    case "content":
      return "Search file contents";
    case "line":
      return "Go to line";
    case "files":
      return "Files — type > for commands, ? to search contents, : for a line";
  }
}

export function CommandBar({
  candidates,
  placeholder = "Search",
}: {
  /** Answers for a query. Async because searching contents is a request. */
  candidates: (mode: CommandMode, term: string) => Promise<CommandItem[]> | CommandItem[];
  placeholder?: string;
}) {
  const [raw, setRaw] = useState("");
  const [open, setOpen] = useState(false);
  const [items, setItems] = useState<CommandItem[]>([]);
  const [selected, setSelected] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const { mode, term } = useMemo(() => parseQuery(raw), [raw]);

  // ⌘P from anywhere, the chord every editor uses for "go to a thing".
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!(event.metaKey || event.ctrlKey) || event.key.toLowerCase() !== "p") return;
      event.preventDefault();
      setOpen(true);
      setRaw(event.shiftKey ? ">" : "");
      inputRef.current?.focus();
      inputRef.current?.select();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  useEffect(() => {
    if (!open) return;
    let live = true;
    const timer = window.setTimeout(() => {
      void Promise.resolve(candidates(mode, term)).then(
        (found) => {
          if (!live) return;
          setItems(found);
          setSelected(0);
        },
        () => live && setItems([]),
      );
      // A keystroke is not a question; a pause is. Short, because this is the
      // control people type into fastest.
    }, 120);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
  }, [open, mode, term, candidates]);

  const choose = (item: CommandItem | undefined) => {
    if (!item) return;
    setOpen(false);
    setRaw("");
    item.run();
  };

  return (
    <div className="command-bar" role="search">
      <input
        ref={inputRef}
        className="command-bar__input"
        type="text"
        value={raw}
        placeholder={placeholder}
        aria-label={modeHint(mode)}
        aria-expanded={open}
        onFocus={() => setOpen(true)}
        // A blur that closes immediately would fire before the click on a
        // result landed; the frame's delay is what lets a pick through.
        onBlur={() => window.setTimeout(() => setOpen(false), 120)}
        onChange={(event) => {
          setRaw(event.target.value);
          setOpen(true);
        }}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            setOpen(false);
            inputRef.current?.blur();
          } else if (event.key === "ArrowDown") {
            event.preventDefault();
            setSelected((i) => Math.min(i + 1, items.length - 1));
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
              {term ? "Nothing matches." : "Start typing."}
            </p>
          ) : (
            <ul className="command-bar__list" role="listbox">
              {items.map((item, index) => (
                <li key={item.id}>
                  <button
                    type="button"
                    role="option"
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
  );
}
