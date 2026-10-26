<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/InsertMenu.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c9a0adfc5a83"></a>
## `apps/web/src/components/InsertMenu.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A write exposes its proposed effects](../../../../../guarantees/authoring/a-write-exposes-its-proposed-effects/contract.md), [Inserting An Element Writes Hick You Could Have Typed Yourself](../../../../../guarantees/authoring/inserting-an-element-writes-hick-you-could-have-typed/contract.md).

<a id="f-c9a0adfc5a83-InsertMenuProps"></a>
### `InsertMenuProps`

<storycode:copy id="f-c9a0adfc5a83-InsertMenuProps">
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiActor, UiCommand, UiOutcome } from "../lib/uiActions";
import { ChangePreview } from "./ChangePreview";
// The Insert menu: the hick vocabulary, as something you pick rather than
// something you remember.
//
// Two panes, both always visible. On the left every element the catalogue
// knows (lib/insertCatalog.ts), searchable and arrow-navigable. On the right
// the chosen element's attributes as real fields — each labelled, each with
// the sentence from the language guide that says what it is for, required
// ones marked — its body, and underneath, the exact bytes the insert will
// write.
//
// The preview is the point. This is a text format a person owns and edits by
// hand; a menu that writes XML you never see teaches you nothing and leaves
// you unable to fix what it wrote. Seeing the tag assemble as you fill the
// form is how the second one gets typed straight into the buffer.
//
// Nothing here touches an editor. Picking Insert hands the element and its
// values back to the workspace, which owns the question of which buffer is
// being written to (editor/activeEditor.ts).

import { useEffect, useId, useMemo, useRef, useState } from "react";

import {
  INSERT_GROUPS,
  defaultBody,
  elementById,
  filterElements,
  initialValues,
  renderElement,
  validate,
  type FieldValues,
  type InsertElement,
} from "../lib/insertCatalog";

export interface InsertMenuProps {
  /** Preselected element, from a native-menu pick. */
  initialId?: string | null;
  /** What is selected in the buffer right now — becomes the body. */
  selectedText: string;
  /** Insert this. The workspace writes it into the focused buffer. */
  onInsert: (element: InsertElement, values: FieldValues, body: string, actor?: UiActor) => void | boolean;
  sourceCurrent?: () => boolean;
  /** Editing fixes the element instead of offering the vocabulary, and seeds
   * the fields from the element already in the document. */
  edit?: { values: FieldValues; body: string };
  onClose: () => void;
}

</storycode:copy>
<a id="f-c9a0adfc5a83-InsertMenu"></a>
### `InsertMenu`

<storycode:copy id="f-c9a0adfc5a83-InsertMenu">
export function InsertMenu({
  initialId,
  selectedText,
  onInsert,
  edit,
  sourceCurrent,
  onClose,
}: InsertMenuProps) {
  const parent = `insert-form:${useId()}`;
  const [formError, setFormError] = useState("");
  const [query, setQuery] = useState("");
  const matches = useMemo(() => filterElements(query), [query]);

  // The chosen element. A native-menu pick lands here already made; typing a
  // query that excludes the choice moves it to the best remaining match, so
  // the form is never showing something the list no longer offers.
  const [chosenId, setChosenId] = useState<string>(
    () =>
      (initialId && elementById(initialId) ? initialId : null) ??
      filterElements("")[0].id,
  );
  useEffect(() => {
    if (matches.length > 0 && !matches.some((m) => m.id === chosenId))
      setChosenId(matches[0].id);
  }, [matches, chosenId]);
  const element = elementById(chosenId) ?? matches[0];

  // One set of values per element, remade when the choice changes: an
  // attribute typed for a container means nothing to a paste.
  const [values, setValues] = useState<Record<string, string>>(() =>
    edit?.values ?? initialValues(element),
  );
  const [body, setBody] = useState<string>(() =>
    edit?.body ?? defaultBody(element, selectedText, initialValues(element)),
  );
  const [showProblems, setShowProblems] = useState(false);
  useEffect(() => {
    if (edit) return;
    setValues(initialValues(element));
    setBody(defaultBody(element, selectedText, initialValues(element)));
    setShowProblems(false);
  }, [element, selectedText, edit]);

  /** A field change that decides the starter body (the diagram's renderer)
   * swaps an UNTOUCHED body along with it; a body the person already edited
   * is theirs and stays. */
  const setFieldValue = (name: string, value: string) => {
    setValues((current) => {
      const next = { ...current, [name]: value };
      if (element.bodyPlaceholderBy?.field === name) {
        setBody((b) =>
          b === defaultBody(element, selectedText, current)
            ? defaultBody(element, selectedText, next)
            : b,
        );
      }
      return next;
    });
  };

  const searchRef = useRef<HTMLInputElement>(null);
  const firstFieldRef = useRef<HTMLInputElement | HTMLSelectElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    // A pick from the native menu has already chosen; land in the form.
    if (initialId && elementById(initialId)) firstFieldRef.current?.focus();
    else searchRef.current?.focus();
  }, [initialId]);
  useEffect(() => {
    const chosen = listRef.current?.querySelector("[aria-selected='true']");
    // Guarded: jsdom has no scrollIntoView, and keeping the row visible is
    // not worth a component that cannot be rendered in a test.
    if (
      chosen instanceof HTMLElement &&
      typeof chosen.scrollIntoView === "function"
    ) {
      chosen.scrollIntoView({ block: "nearest" });
    }
  }, [chosenId]);

  const problems = validate(element, values);
  const blocked = Object.keys(problems).length > 0;

  const preview = useMemo(
    () => renderElement(element, values, body),
    [element, values, body],
  );
  // What the padding rules will do to it, described rather than drawn — the
  // preview shows the element, this line says where it goes.
  const placementNote =
    element.placement === "block"
      ? "Goes on its own, with a blank line above and below."
      : element.placement === "child"
        ? `Goes on its own line inside ${element.belongsIn ? `a ${element.belongsIn}` : "its parent"}, indented.`
        : "Goes inline, right where the caret is.";

  const current = () => sourceCurrent?.() ?? true;
  const submit = (actor: UiActor): UiOutcome => {
    if (blocked) { setShowProblems(true); return { status: "unavailable", reason: "Fill the required attributes first" }; }
    if (!current()) { setFormError("The originating editor changed, closed or became read-only. Reopen this form."); return { status: "unavailable", reason: "The captured source is no longer current" }; }
    const result = onInsert(element, values, body, actor);
    if (result === false) { setFormError("The originating editor refused the change. Inspect its current source and reopen the form."); return { status: "unavailable", reason: "Source change was refused by its actual editor" }; }
    onClose();
    return { status: "completed", value: { editor_handler_returned: true, source_write: result === true ? "dispatched" : "Inspect actual source; callback supplied no write acknowledgement", actor, saved: "not established", verification: "not requested" } };
  };
  const revision = useUiRevision(() => [query, element.id, values, body, formError, current(), edit?.values, edit?.body]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], execute: UiCommand["execute"], enabled = true) => commands.push({ key: `${parent}:${key}`, command: { label, parent, subject: parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled: enabled && (key !== "submit" || current()), reason: !current() ? "Originating editor/source changed; reopen the form" : !enabled ? "Fill required attributes" : undefined }), execute } });
  if (!edit) {
    add("query", "Search insert element catalogue", { query: { type: "string", maxLength: 16384 } }, async args => { setQuery(args.query as string); return { status: "completed", value: { query_changed: true } }; });
    add("choose", "Select insert element", { id: { type: "string", maxLength: 4096 } }, async args => {
      if (!matches.some(candidate => candidate.id === args.id)) return { status: "unavailable", reason: "Observe a current matching catalogue identity" };
      setChosenId(args.id as string); return { status: "completed", value: { selected: args.id } };
    });
  }
  for (const field of element.fields) add(`field:${field.name}`, `Set element attribute: ${field.label}`, { value: { type: "string", maxLength: 1048576, ...(field.choices ? { choices: [...field.choices] } : {}) } }, async args => { setFieldValue(field.name, args.value as string); return { status: "completed", value: { draft_changed: true, source_written: false } }; });
  if (element.body !== "none") add("body", "Set element body draft", { body: { type: "string", maxLength: 1048576 } }, async args => { setBody(args.body as string); return { status: "completed", value: { draft_changed: true, source_written: false } }; });
  add("submit", edit ? "Apply captured element edit" : "Insert into captured source", {}, async (_args, actor) => submit(actor), !blocked);
  add("cancel", "Cancel element form", {}, async () => { onClose(); return { status: "completed", value: { closed: true, source_written: false } }; });
  add("read", "Read element form bytes", { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ mode: edit ? "edit" : "insert", query, matches: edit ? [] : matches.map(m => ({ id: m.id, title: m.title, summary: m.summary, group: m.group })), element, values, body, preview, placement: placementNote, problems, source_current: current(), error: formError, edit_baseline: false }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside current element form" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  });
  const invoke = useUiCommands(commands);
  const click = (key: string) => { if (key === "submit" && blocked) { setShowProblems(true); return; } void invoke(`${parent}:${key}`).catch(reason => setFormError(String(reason))); };
  const move = (delta: number) => {
    const index = matches.findIndex((m) => m.id === chosenId);
    const next =
      matches[Math.min(matches.length - 1, Math.max(0, index + delta))];
    if (next) setChosenId(next.id);
  };

  const onKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      click("cancel");
      return;
    }
    // Insert from anywhere in the panel, including a body textarea where a
    // bare Enter is a newline.
    if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
      event.preventDefault();
      click("submit");
    }
  };

  const onSearchKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      move(1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      move(-1);
    } else if (event.key === "Enter" && !event.metaKey && !event.ctrlKey) {
      // Enter from the list means "this one" — the form is where the values
      // are decided, so it takes the focus rather than inserting blind.
      event.preventDefault();
      firstFieldRef.current?.focus();
    }
  };

  return (
    <div className="insert-menu-backdrop" onMouseDown={() => click("cancel")}>
      <div
        className="insert-menu"
        role="dialog"
        aria-label={edit ? "Edit a hick element" : "Insert a hick element"}
        onMouseDown={(event) => event.stopPropagation()}
        onKeyDown={onKeyDown}
      >
        {formError && <p role="alert">{formError}</p>}
        {!current() && <p role="status">The originating source changed. Cancel and reopen this form.</p>}
        {!edit && <div className="insert-menu__list-pane">
          <input
            ref={searchRef}
            className="insert-menu__search"
            type="search"
            placeholder="Insert…"
            value={query}
            aria-label="Find an element"
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={onSearchKeyDown}
          />
          <div
            className="insert-menu__list"
            ref={listRef}
            role="listbox"
            aria-label="Elements"
          >
            {matches.length === 0 && (
              <p className="insert-menu__empty">
                Nothing by that name. The whole vocabulary is in the language
                guide —<span className="mono"> docs/docs/hick-guide.md</span>.
              </p>
            )}
            {INSERT_GROUPS.map((group) => {
              const inGroup = matches.filter((m) => m.group === group);
              if (inGroup.length === 0) return null;
              return (
                <div key={group} className="insert-menu__group">
                  <h3 className="insert-menu__group-title">{group}</h3>
                  {inGroup.map((candidate) => (
                    <button
                      key={candidate.id}
                      type="button"
                      role="option"
                      aria-selected={candidate.id === element.id}
                      className={`insert-menu__item${candidate.id === element.id ? " on" : ""}`}
                      onClick={() => setChosenId(candidate.id)}
                      onDoubleClick={() => click("submit")}
                    >
                      <span className="insert-menu__item-title">
                        {candidate.title}
                      </span>
                      <span className="insert-menu__item-tag mono">
                        hick:{candidate.tag}
                      </span>
                      <span className="insert-menu__item-summary">
                        {candidate.summary}
                      </span>
                    </button>
                  ))}
                </div>
              );
            })}
          </div>
        </div>}

        <div className="insert-menu__form-pane">
          {/* Only the attributes scroll. The preview and the Insert button
              are pinned below them: an element with seven attributes must
              not push the thing you came to press off the bottom. */}
          <div className="insert-menu__form-scroll">
            <header className="insert-menu__header">
              <h2 className="insert-menu__title">{element.title}</h2>
              <code className="insert-menu__tag">
                &lt;hick:{element.tag}&gt;
              </code>
            </header>
            <p className="insert-menu__summary">{element.summary}</p>
            {element.belongsIn && (
              <p className="insert-menu__belongs">
                Belongs inside a{" "}
                <span className="mono">hick:{element.belongsIn}</span> — put the
                caret in one before inserting.
              </p>
            )}

            <div className="insert-menu__fields">
              {element.fields.map((field, index) => {
                const id = `insert-field-${element.id}-${field.name}`;
                const problem = showProblems ? problems[field.name] : undefined;
                return (
                  <div key={field.name} className="insert-menu__field">
                    <label className="insert-menu__label" htmlFor={id}>
                      {field.label}
                      <span className="mono insert-menu__attr">
                        {field.name}
                      </span>
                      {field.required && (
                        <span
                          className="insert-menu__required"
                          aria-label="required"
                        >
                          required
                        </span>
                      )}
                    </label>
                    {field.choices ? (
                      <select
                        id={id}
                        ref={
                          index === 0
                            ? (firstFieldRef as React.RefObject<HTMLSelectElement>)
                            : undefined
                        }
                        className="insert-menu__input"
                        value={values[field.name] ?? ""}
                        onChange={(event) => setFieldValue(field.name, event.target.value)}
                      >
                        {field.choices.map((choice) => (
                          <option key={choice} value={choice}>
                            {choice === "" ? "(default)" : choice}
                          </option>
                        ))}
                      </select>
                    ) : (
                      <input
                        id={id}
                        ref={
                          index === 0
                            ? (firstFieldRef as React.RefObject<HTMLInputElement>)
                            : undefined
                        }
                        className={`insert-menu__input${problem ? " bad" : ""}`}
                        type="text"
                        value={values[field.name] ?? ""}
                        placeholder={field.placeholder}
                        spellCheck={false}
                        onChange={event => setFieldValue(field.name, event.target.value)}
                      />
                    )}
                    <p className="insert-menu__hint">{field.hint}</p>
                    {problem && (
                      <p className="insert-menu__problem" role="alert">
                        {problem}
                      </p>
                    )}
                  </div>
                );
              })}

              {element.body !== "none" && (
                <div className="insert-menu__field">
                  <label
                    className="insert-menu__label"
                    htmlFor={`insert-body-${element.id}`}
                  >
                    {element.bodyLabel}
                    {!edit && selectedText.trim().length > 0 && (
                      <span className="insert-menu__attr">
                        from your selection
                      </span>
                    )}
                  </label>
                  <textarea
                    id={`insert-body-${element.id}`}
                    className="insert-menu__input insert-menu__body mono"
                    rows={element.body === "inline" ? 1 : 4}
                    value={body}
                    spellCheck={false}
                    onChange={(event) => setBody(event.target.value)}
                  />
                </div>
              )}
            </div>
          </div>

          <ChangePreview heading={edit ? "What changes" : "What gets written"} effects={edit ? "Replaces this element in its captured document; unknown attributes remain intact." : placementNote}>
            <pre className="insert-menu__preview-text mono">{preview}</pre>
          </ChangePreview>

          <div className="insert-menu__actions">
            <span className="insert-menu__shortcut">
              {navigator.platform.startsWith("Mac") ? "⌘" : "Ctrl"}+Enter
            </span>
            <button type="button" className="btn" onClick={() => click("cancel")}>
              Cancel
            </button>
            <button
              type="button"
              className="btn btn-primary"
              onClick={() => click("submit")}
              data-tip={
                blocked ? "Fill the required attributes first" : undefined
              }
            >
              {edit ? "Apply" : "Insert"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/InsertMenu.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/InsertMenu.tsx"><storycode:paste select="#f-c9a0adfc5a83-InsertMenuProps"/><storycode:paste select="#f-c9a0adfc5a83-InsertMenu"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-d0524ef3ebdd"></a>
## `apps/web/src/components/InsertMenu.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [A write exposes its proposed effects](../../../../../guarantees/authoring/a-write-exposes-its-proposed-effects/contract.md), [Inserting An Element Writes Hick You Could Have Typed Yourself](../../../../../guarantees/authoring/inserting-an-element-writes-hick-you-could-have-typed/contract.md).

<a id="f-d0524ef3ebdd-expression_statement"></a>
### `expression_statement`

@vitest-environment jsdom
Protects docs/guarantees/authoring/inserting-an-element-writes-hick-you-could-have-typed.md

<storycode:copy id="f-d0524ef3ebdd-expression_statement">
// @vitest-environment jsdom
//
// Protects docs/guarantees/authoring/inserting-an-element-writes-hick-you-could-have-typed.md

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EditorState, StateEffect, Transaction } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { InsertMenu } from "./InsertMenu";
import { insertElement } from "../editor/insertElement";
import { elementById, initialValues, type InsertElement } from "../lib/insertCatalog";
import {
  activeEditor,
  forgetEditor,
  markActiveEditor,
  resetActiveEditor,
} from "../editor/activeEditor";

afterEach(() => {
  cleanup();
  resetActiveEditor();
});

</storycode:copy>
<a id="f-d0524ef3ebdd-panel"></a>
### `panel`

<storycode:copy id="f-d0524ef3ebdd-panel">
function panel(props: Partial<React.ComponentProps<typeof InsertMenu>> = {}) {
  const onInsert = vi.fn();
  const onClose = vi.fn();
  render(<InsertMenu selectedText="" onInsert={onInsert} onClose={onClose} {...props} />);
  return { onInsert, onClose };
}

</storycode:copy>
<a id="f-d0524ef3ebdd-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d0524ef3ebdd-expression_statement-2">
describe("choosing an element", () => {
  it("shows the whole vocabulary, grouped", () => {
    panel();
    expect(screen.getByRole("heading", { name: "Execution" })).toBeTruthy();
    expect(screen.getByRole("option", { name: /Exec Cell/i })).toBeTruthy();
    expect(screen.getByRole("option", { name: /^Paste hick:paste/i })).toBeTruthy();
  });

  it("opens on the element a native menu item named", () => {
    panel({ initialId: "allow-network" });
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe("Allow network");
  });

  it("falls back to the first element when the shell names one it does not have", () => {
    // A newer desktop menu against an older page: the panel opens, which is
    // a better answer than a menu item that does nothing.
    panel({ initialId: "teleport" });
    expect(screen.getByRole("heading", { level: 2 })).toBeTruthy();
  });

  it("narrows the list as you type, and moves the choice with it", () => {
    panel();
    fireEvent.change(screen.getByLabelText("Find an element"), { target: { value: "network" } });
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe("Allow network");
    expect(screen.queryByRole("option", { name: /Exec Cell/i })).toBeNull();
  });

  it("says so when nothing matches, and points at the guide", () => {
    panel();
    fireEvent.change(screen.getByLabelText("Find an element"), { target: { value: "zzz" } });
    expect(screen.getByText(/hick-guide/)).toBeTruthy();
  });
});

</storycode:copy>
<a id="f-d0524ef3ebdd-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-d0524ef3ebdd-expression_statement-3">
describe("filling it in", () => {
  it("previews the exact bytes as the fields change", () => {
    panel({ initialId: "copy" });
    fireEvent.change(screen.getByLabelText(/^Id/), { target: { value: "version" } });
    expect(screen.getByText('<hick:copy id="version">', { exact: false })).toBeTruthy();
  });

  it("names each attribute beside its label, so the next one can be typed by hand", () => {
    panel({ initialId: "exec" });
    expect(screen.getByText("container")).toBeTruthy();
    expect(screen.getByText("timeout")).toBeTruthy();
  });

  it("seeds the body from the buffer's selection", () => {
    panel({ initialId: "cut", selectedText: "a secret paragraph" });
    expect((screen.getByLabelText(/Content/) as HTMLTextAreaElement).value).toBe(
      "a secret paragraph",
    );
  });

  it("swaps the diagram's starter body with its renderer — mermaid text or a scene", () => {
    panel({ initialId: "diagram" });
    const bodyInput = () => screen.getByLabelText(/Diagram source/) as HTMLTextAreaElement;
    expect(bodyInput().value).toContain("graph TD");
    // Choosing the graph renderer replaces an UNTOUCHED starter with a scene
    // the canvas can open — mermaid text under renderer="graph" would greet
    // the author with a parse error.
    fireEvent.change(screen.getByLabelText(/Renderer/), { target: { value: "graph" } });
    expect(bodyInput().value).toContain('"nodes"');
    fireEvent.change(screen.getByLabelText(/Renderer/), { target: { value: "mermaid" } });
    expect(bodyInput().value).toContain("graph TD");
    // A body the person already edited is theirs, whatever the renderer says.
    fireEvent.change(bodyInput(), { target: { value: "flowchart LR\n  a --> b" } });
    fireEvent.change(screen.getByLabelText(/Renderer/), { target: { value: "graph" } });
    expect(bodyInput().value).toBe("flowchart LR\n  a --> b");
  });

  it("refuses to insert while a required attribute is empty, and says which", () => {
    const { onInsert } = panel({ initialId: "copy" });
    fireEvent.click(screen.getByRole("button", { name: "Insert" }));
    expect(onInsert).not.toHaveBeenCalled();
    expect(screen.getByRole("alert").textContent).toContain("Copy needs an id");
  });

  it("hands back the element and its values, then closes", () => {
    const { onInsert, onClose } = panel({ initialId: "paste" });
    fireEvent.change(screen.getByLabelText(/^Select/), { target: { value: "version" } });
    fireEvent.click(screen.getByRole("button", { name: "Insert" }));
    expect(onInsert).toHaveBeenCalledTimes(1);
    const [element, values] = onInsert.mock.calls[0];
    expect(element.id).toBe("paste");
    expect(values.select).toBe("version");
    expect(onClose).toHaveBeenCalled();
  });

  it("edits one existing element without showing the element picker", () => {
    const { onInsert } = panel({
      initialId: "copy",
      edit: { values: { id: "draft" }, body: "Existing text." },
    });
    expect(screen.getByRole("dialog", { name: "Edit a hick element" })).toBeTruthy();
    expect(screen.queryByLabelText("Find an element")).toBeNull();
    expect((screen.getByLabelText(/^Id/) as HTMLInputElement).value).toBe("draft");
    expect((screen.getByLabelText(/Content/) as HTMLTextAreaElement).value).toBe("Existing text.");
    fireEvent.change(screen.getByLabelText(/^Id/), { target: { value: "published" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    expect(onInsert.mock.calls[0][1].id).toBe("published");
  });

  it("closes on Escape without inserting", () => {
    const { onInsert, onClose } = panel();
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
    expect(onInsert).not.toHaveBeenCalled();
  });
});

</storycode:copy>
<a id="f-d0524ef3ebdd-views"></a>
### `views`

The other half: which buffer it lands in, and what lands there.

<storycode:copy id="f-d0524ef3ebdd-views">
// ---------------------------------------------------------------------------
// The other half: which buffer it lands in, and what lands there.
// ---------------------------------------------------------------------------

const views: EditorView[] = [];
</storycode:copy>
<a id="f-d0524ef3ebdd-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-d0524ef3ebdd-expression_statement-4">
afterEach(() => {
  for (const view of views.splice(0)) view.destroy();
});

</storycode:copy>
<a id="f-d0524ef3ebdd-editor"></a>
### `editor`

<storycode:copy id="f-d0524ef3ebdd-editor">
function editor(doc: string): EditorView {
  const view = new EditorView({ parent: document.body, state: EditorState.create({ doc }) });
  views.push(view);
  return view;
}

</storycode:copy>
<a id="f-d0524ef3ebdd-insert"></a>
### `insert`

<storycode:copy id="f-d0524ef3ebdd-insert">
function insert(view: EditorView, id: string, values: Record<string, string>, body?: string) {
  const element = elementById(id) as InsertElement;
  insertElement(view, element, { ...initialValues(element), ...values }, body);
}

</storycode:copy>
<a id="f-d0524ef3ebdd-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-d0524ef3ebdd-expression_statement-5">
describe("the target buffer", () => {
  it("is the one focused last, not the one focused now", () => {
    // The panel takes the focus the moment it opens, so "has focus" is
    // already the wrong question by the time an insert happens.
    const first = editor("one");
    const second = editor("two");
    markActiveEditor(first);
    markActiveEditor(second);
    expect(activeEditor()).toBe(second);
  });

  it("is forgotten when its pane goes away", () => {
    const view = editor("one");
    markActiveEditor(view);
    forgetEditor(view);
    expect(activeEditor()).toBeNull();
  });

  it("keeps the newer target when an older pane closes behind it", () => {
    const old = editor("one");
    const current = editor("two");
    markActiveEditor(old);
    markActiveEditor(current);
    forgetEditor(old);
    expect(activeEditor()).toBe(current);
  });

  it("is not a destroyed editor, which would swallow the edit in silence", () => {
    const view = editor("one");
    markActiveEditor(view);
    view.destroy();
    expect(activeEditor()).toBeNull();
  });
});

</storycode:copy>
<a id="f-d0524ef3ebdd-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-d0524ef3ebdd-expression_statement-6">
describe("what lands in the buffer", () => {
  it("writes the element where the caret is", () => {
    const view = editor("Version: ");
    view.dispatch({ selection: { anchor: 9 } });
    insert(view, "paste", { select: "version" });
    expect(view.state.doc.toString()).toBe('Version: <hick:paste select="#version" />');
  });

  it("leaves the body selected so the first keystroke replaces it", () => {
    const view = editor("");
    insert(view, "exec", { container: "py" });
    const { from, to } = view.state.selection.main;
    expect(view.state.sliceDoc(from, to)).toBe("echo hello");
  });

  it("wraps the selection when there is one, and opens the block a line", () => {
    const view = editor("keep this\n");
    view.dispatch({ selection: { anchor: 0, head: 9 } });
    insert(view, "copy", { id: "kept" }, "keep this");
    expect(view.state.doc.toString()).toBe('<hick:copy id="kept">\nkeep this\n</hick:copy>\n\n');
  });

  it("arrives as something the person did, so undo can reach it", () => {
    // DocumentEditor keeps every transaction WITHOUT a userEvent out of the
    // undo history — that is how a collaborator's edits and the room's first
    // sync stay un-undoable. An insert dispatched without one would be
    // permanent.
    const seen: (string | undefined)[] = [];
    const view = new EditorView({
      parent: document.body,
      state: EditorState.create({
        doc: "",
        extensions: [
          EditorView.updateListener.of((update) => {
            for (const tr of update.transactions) {
              if (tr.docChanged) seen.push(tr.annotation(Transaction.userEvent));
            }
          }),
        ],
      }),
    });
    views.push(view);
    insert(view, "confirm", { message: "Go?" });
    expect(seen).toEqual(["input.insertElement"]);
  });
});

</storycode:copy>
<a id="f-d0524ef3ebdd-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-d0524ef3ebdd-expression_statement-7">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("keeps a captured editor/range across focus changes and rejects changed/readonly source", async () => {
  const { captureInsertion, insertCapturedElement } = await import("../editor/insertElement");
  const first = editor("original selection"), other = editor("other buffer");
  first.dispatch({ selection: { anchor: 0, head: 8 } });
  const capture = captureInsertion(first);
  const events: (string | undefined)[] = [];
  first.dispatch({ effects: StateEffect.appendConfig.of(EditorView.updateListener.of(update => update.transactions.forEach(tr => events.push(tr.annotation(Transaction.userEvent))))) });
  markActiveEditor(other);
  first.dispatch({ selection: { anchor: first.state.doc.length } });
  const element = elementById("paste")!;
  expect(insertCapturedElement(capture, element, { ...initialValues(element), select: "known" }, "", true)).toBe(true);
  expect(first.state.doc.toString()).toContain('select="#known"');
  expect(first.state.doc.toString()).not.toContain("original");
  expect(other.state.doc.toString()).toBe("other buffer");
  expect(events).toContain("input.uiAgentInsert");
  expect(insertCapturedElement(capture, element, { select: "old" }, "", true)).toBe(false);
  const readonly = new EditorView({ parent: document.body, state: EditorState.create({ doc: "readonly", extensions: [EditorState.readOnly.of(true)] }) });
  views.push(readonly);
  expect(insertCapturedElement(captureInsertion(readonly), element, { select: "no" }, "", true)).toBe(false);
});
</storycode:copy>
<a id="f-d0524ef3ebdd-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-d0524ef3ebdd-expression_statement-8">
it("drives typed element form actions through the real callback and refuses stale source", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const { act } = await import("@testing-library/react");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const onInsert = vi.fn(() => true), onClose = vi.fn(); let current = true;
  render(<UiActionContext.Provider value={registry}><InsertMenu initialId="copy" selectedText="PRIVATE selected λ" onInsert={onInsert} onClose={onClose} sourceCurrent={() => current} /></UiActionContext.Provider>);
  const parent = registry.groups().groups.find(g => g.startsWith("insert-form:"))!;
  expect(JSON.stringify(registry.observe(parent))).not.toContain("PRIVATE selected λ");
  const attribute = registry.observe(parent).actions.find(a => a.label === "Set element attribute: Id")!;
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: attribute.handle, arguments: { value: "claim" }, request_id: "id" }); });
  const submit = registry.observe(parent).actions.find(a => a.label === "Insert into captured source")!;
  current = false;
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: submit.handle, arguments: {}, request_id: "stale" })).toMatchObject({ status: "stale" });
  expect(onInsert).not.toHaveBeenCalled(); expect(onClose).not.toHaveBeenCalled();
  current = true;
  const fresh = registry.observe(parent).actions.find(a => a.label === "Insert into captured source")!;
  await act(async () => { expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: fresh.handle, arguments: {}, request_id: "insert" })).toMatchObject({ outcome: { value: { source_write: "dispatched", actor: { kind: "agent" }, verification: "not requested" } } }); });
  expect(onInsert).toHaveBeenCalledWith(expect.objectContaining({ id: "copy" }), expect.objectContaining({ id: "claim" }), "PRIVATE selected λ", { id: "python:turn", kind: "agent" });
  expect(onClose).toHaveBeenCalledOnce();
});
</storycode:copy>

### Assemble `apps/web/src/components/InsertMenu.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/InsertMenu.test.tsx"><storycode:paste select="#f-d0524ef3ebdd-expression_statement"/><storycode:paste select="#f-d0524ef3ebdd-panel"/><storycode:paste select="#f-d0524ef3ebdd-expression_statement-2"/><storycode:paste select="#f-d0524ef3ebdd-expression_statement-3"/><storycode:paste select="#f-d0524ef3ebdd-views"/><storycode:paste select="#f-d0524ef3ebdd-expression_statement-4"/><storycode:paste select="#f-d0524ef3ebdd-editor"/><storycode:paste select="#f-d0524ef3ebdd-insert"/><storycode:paste select="#f-d0524ef3ebdd-expression_statement-5"/><storycode:paste select="#f-d0524ef3ebdd-expression_statement-6"/><storycode:paste select="#f-d0524ef3ebdd-expression_statement-7"/><storycode:paste select="#f-d0524ef3ebdd-expression_statement-8"/></storycode:file>


</storycode:doc>
