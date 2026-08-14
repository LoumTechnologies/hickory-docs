// A small in-app prompt, for the two places the language server needs an
// answer before it can act: a new name for a rename, and which code action
// to apply.
//
// Deliberately NOT `window.prompt`. That call blocks the whole webview while
// it is open — including the CRDT sync and any run in flight — and Tauri's
// webview does not implement it on every platform, so the desktop build would
// silently do nothing where the browser build worked.

import { useCallback, useRef, useState } from "react";

interface TextPrompt {
  kind: "text";
  title: string;
  initial: string;
  resolve: (value: string | null) => void;
}

interface ChoicePrompt<T> {
  kind: "choice";
  title: string;
  options: { label: string; value: T }[];
  resolve: (value: T | null) => void;
}

type Prompt = TextPrompt | ChoicePrompt<unknown>;

export function usePrompt() {
  const [prompt, setPrompt] = useState<Prompt | null>(null);
  // The resolver lives in a ref as well as in state: unmounting mid-prompt
  // must settle the promise rather than leave the caller awaiting forever.
  const pending = useRef<Prompt | null>(null);

  const askText = useCallback((title: string, initial: string): Promise<string | null> => {
    return new Promise((resolve) => {
      const next: TextPrompt = { kind: "text", title, initial, resolve };
      pending.current = next;
      setPrompt(next);
    });
  }, []);

  const askChoice = useCallback(
    <T,>(title: string, options: { label: string; value: T }[]): Promise<T | null> => {
      return new Promise((resolve) => {
        const next = {
          kind: "choice" as const,
          title,
          options,
          resolve: resolve as (value: unknown) => void,
        };
        pending.current = next;
        setPrompt(next);
      });
    },
    [],
  );

  const settle = useCallback((value: unknown) => {
    const current = pending.current;
    pending.current = null;
    setPrompt(null);
    current?.resolve(value as never);
  }, []);

  return { prompt, askText, askChoice, settle };
}

export function PromptPanel({
  prompt,
  onSettle,
}: {
  prompt: ReturnType<typeof usePrompt>["prompt"];
  onSettle: (value: unknown) => void;
}) {
  const [draft, setDraft] = useState("");
  const shownFor = useRef<Prompt | null>(null);

  if (!prompt) return null;
  // Reset the draft when a new prompt opens, without an effect: rendering a
  // different prompt is the only thing that should clear what was typed.
  if (shownFor.current !== prompt) {
    shownFor.current = prompt;
    if (prompt.kind === "text") setDraft(prompt.initial);
  }

  return (
    <div className="prompt-panel" role="dialog" aria-label={prompt.title}>
      <p className="prompt-panel__title">{prompt.title}</p>
      {prompt.kind === "text" ? (
        <form
          onSubmit={(event) => {
            event.preventDefault();
            onSettle(draft.trim() ? draft : null);
          }}
        >
          {/* eslint-disable-next-line jsx-a11y/no-autofocus */}
          <input
            autoFocus
            value={draft}
            aria-label={prompt.title}
            onChange={(event) => setDraft(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") onSettle(null);
            }}
          />
          <button type="submit">Apply</button>
          <button type="button" onClick={() => onSettle(null)}>
            Cancel
          </button>
        </form>
      ) : (
        <ul className="prompt-panel__options">
          {prompt.options.map((option, index) => (
            <li key={`${option.label}-${index}`}>
              <button type="button" onClick={() => onSettle(option.value)}>
                {option.label}
              </button>
            </li>
          ))}
          <li>
            <button type="button" onClick={() => onSettle(null)}>
              Cancel
            </button>
          </li>
        </ul>
      )}
    </div>
  );
}
