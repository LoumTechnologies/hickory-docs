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

import { useEffect, useMemo, useRef, useState } from "react";

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
  onInsert: (element: InsertElement, values: FieldValues, body: string) => void;
  onClose: () => void;
}

export function InsertMenu({
  initialId,
  selectedText,
  onInsert,
  onClose,
}: InsertMenuProps) {
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
    initialValues(element),
  );
  const [body, setBody] = useState<string>(() =>
    defaultBody(element, selectedText),
  );
  const [showProblems, setShowProblems] = useState(false);
  useEffect(() => {
    setValues(initialValues(element));
    setBody(defaultBody(element, selectedText));
    setShowProblems(false);
  }, [element, selectedText]);

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

  const submit = () => {
    if (blocked) {
      setShowProblems(true);
      return;
    }
    onInsert(element, values, body);
    onClose();
  };

  const move = (delta: number) => {
    const index = matches.findIndex((m) => m.id === chosenId);
    const next =
      matches[Math.min(matches.length - 1, Math.max(0, index + delta))];
    if (next) setChosenId(next.id);
  };

  const onKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
      return;
    }
    // Insert from anywhere in the panel, including a body textarea where a
    // bare Enter is a newline.
    if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
      event.preventDefault();
      submit();
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
    <div className="insert-menu-backdrop" onMouseDown={onClose}>
      <div
        className="insert-menu"
        role="dialog"
        aria-label="Insert a hick element"
        onMouseDown={(event) => event.stopPropagation()}
        onKeyDown={onKeyDown}
      >
        <div className="insert-menu__list-pane">
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
                      onDoubleClick={submit}
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
        </div>

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
                        onChange={(event) =>
                          setValues((v) => ({
                            ...v,
                            [field.name]: event.target.value,
                          }))
                        }
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
                        onChange={(event) =>
                          setValues((v) => ({
                            ...v,
                            [field.name]: event.target.value,
                          }))
                        }
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
                    {selectedText.trim().length > 0 && (
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

          <div className="insert-menu__preview">
            <h3 className="insert-menu__preview-title">What gets written</h3>
            <pre className="insert-menu__preview-text mono">{preview}</pre>
            <p className="insert-menu__hint">{placementNote}</p>
          </div>

          <div className="insert-menu__actions">
            <span className="insert-menu__shortcut">
              {navigator.platform.startsWith("Mac") ? "⌘" : "Ctrl"}+Enter
            </span>
            <button type="button" className="btn" onClick={onClose}>
              Cancel
            </button>
            <button
              type="button"
              className="btn btn-primary"
              onClick={submit}
              data-tip={
                blocked ? "Fill the required attributes first" : undefined
              }
            >
              Insert
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
