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
