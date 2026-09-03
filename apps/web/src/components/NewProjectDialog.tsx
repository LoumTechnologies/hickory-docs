// File → New Project: `dotnet new`, as a form.
//
// Three panes with one idea between them. On the left every template this
// machine's SDK has, searchable. In the middle the chosen template's own
// options, read out of `dotnet new <template> --help` rather than out of a
// list we maintain — a template from a NuGet package this app has never heard
// of gets the same form as `console`. Underneath, the exact commit that is
// about to be written.
//
// The preview is not decoration, and it is the same argument the Insert panel
// makes: this is a text format a person owns and edits by hand, so a dialog
// that writes markup you never see teaches you nothing and leaves you unable
// to fix what it wrote. It comes from the server, from the function that
// writes the file, so it cannot be a picture of a different document.
//
// What "New Project" means here is not "make me a folder". It is: run the
// scaffolder, and **ingest** what it wrote, so the forty files it produced are
// bytes this document owns and a clone rebuilds without the SDK.
// docs/specs/freeform/owning-what-a-scaffolder-wrote.md

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { ApiError, api } from "../api/client";
import type {
  ScaffoldCatalog,
  ScaffoldCreated,
  ScaffoldSpec,
  ScaffoldTemplate,
  ScaffoldTemplateDetail,
  TemplateOption,
} from "../api/types";
import {
  NO_RESTORE,
  chosenOptions,
  filterTemplates,
  grouped,
  initialLanguage,
  initialValues,
  problems as validate,
  slug,
  type FieldValues,
} from "../lib/scaffold";

export interface NewProjectDialogProps {
  /** The scaffold landed as a commit carrying its recipe. */
  onCreated: (created: ScaffoldCreated) => void;
  onClose: () => void;
}

/** How long to sit still before asking the server what the commit looks
 * like. Long enough that typing a name is one request rather than eight,
 * short enough that the preview feels like it is following you. */
const PREVIEW_DEBOUNCE_MS = 180;

export function NewProjectDialog({ onCreated, onClose }: NewProjectDialogProps) {
  const [catalog, setCatalog] = useState<ScaffoldCatalog | null>(null);
  /** Set when this machine has no SDK — a different screen, not an error
   * line. Keyed off the response's `missing` field rather than its wording. */
  const [noSdk, setNoSdk] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    api
      .scaffoldTemplates()
      .then((c) => live && setCatalog(c))
      .catch((e: unknown) => {
        if (!live) return;
        const missing =
          e instanceof ApiError &&
          typeof e.body === "object" &&
          e.body !== null &&
          (e.body as { missing?: string }).missing === "dotnet";
        if (missing) setNoSdk(true);
        else setLoadError(e instanceof Error ? e.message : String(e));
      });
    return () => {
      live = false;
    };
  }, []);

  if (noSdk) return <NoSdkScreen onClose={onClose} />;
  if (loadError)
    return (
      <Shell onClose={onClose} label="New project">
        <div className="new-project__message">
          <h2>Could not read the templates</h2>
          <p className="new-project__error">{loadError}</p>
          <p className="insert-menu__hint">
            `dotnet new list` is what this asks. Running it in a terminal
            usually says more than this can.
          </p>
          <div className="insert-menu__actions">
            <button type="button" className="btn" onClick={onClose}>
              Close
            </button>
          </div>
        </div>
      </Shell>
    );
  if (!catalog)
    return (
      <Shell onClose={onClose} label="New project">
        <div className="new-project__message">
          <p>Reading this machine's `dotnet new` templates…</p>
        </div>
      </Shell>
    );

  return <Chooser catalog={catalog} onCreated={onCreated} onClose={onClose} />;
}

/** The backdrop and the panel, shared by every state this dialog has. */
function Shell({
  children,
  onClose,
  label,
}: {
  children: React.ReactNode;
  onClose: () => void;
  label: string;
}) {
  return (
    <div className="insert-menu-backdrop" onMouseDown={onClose}>
      <div
        className="new-project"
        role="dialog"
        aria-label={label}
        onMouseDown={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.preventDefault();
            onClose();
          }
        }}
      >
        {children}
      </div>
    </div>
  );
}

/**
 * No SDK on this machine.
 *
 * A sentence and a link, not a button: unlike a debug adapter, the .NET SDK
 * is not something this product has a catalogue for or any business fetching
 * — it is a several-hundred-megabyte platform install with its own installer
 * and its own licence. Offering a button we cannot honour would be worse than
 * saying plainly where to get it.
 * docs/guarantees/debugging/a-missing-debugger-is-a-button.md
 */
function NoSdkScreen({ onClose }: { onClose: () => void }) {
  return (
    <Shell onClose={onClose} label="New project">
      <div className="new-project__message">
        <h2>No .NET SDK on this machine</h2>
        <p>
          New Project asks <code>dotnet new</code> what it can scaffold, and
          there is no <code>dotnet</code> on this machine's PATH.
        </p>
        <p className="insert-menu__hint">
          Install it from{" "}
          <span className="mono">https://dotnet.microsoft.com/download</span>,
          then reopen this window — a running process keeps the PATH it was
          started with, so an SDK installed just now will not be visible until
          it restarts.
        </p>
        <div className="insert-menu__actions">
          <button type="button" className="btn" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </Shell>
  );
}

function Chooser({
  catalog,
  onCreated,
  onClose,
}: {
  catalog: ScaffoldCatalog;
} & Omit<NewProjectDialogProps, never>) {
  const [query, setQuery] = useState("");
  const matches = useMemo(
    () => filterTemplates(catalog.templates, query),
    [catalog.templates, query],
  );

  // The chosen template. Typing a query that excludes it moves the choice to
  // the best remaining match, so the form never shows something the list no
  // longer offers.
  const [chosen, setChosen] = useState<string>(
    () =>
      catalog.templates.find((t) => t.short_names.includes("console"))
        ?.short_names[0] ??
      catalog.templates[0]?.short_names[0] ??
      "",
  );
  useEffect(() => {
    if (matches.length > 0 && !matches.some((m) => m.short_names[0] === chosen))
      setChosen(matches[0].short_names[0]);
  }, [matches, chosen]);
  const template: ScaffoldTemplate | undefined =
    catalog.templates.find((t) => t.short_names[0] === chosen) ?? matches[0];

  const [language, setLanguage] = useState("");
  const [detail, setDetail] = useState<ScaffoldTemplateDetail | null>(null);
  const [detailError, setDetailError] = useState<string | null>(null);
  const [values, setValues] = useState<FieldValues>({});

  // A new template means a new language and a new set of fields: an
  // `--auth` typed for `webapi` means nothing to `classlib`.
  useEffect(() => {
    if (!template) return;
    setLanguage(initialLanguage(template));
  }, [template]);

  useEffect(() => {
    if (!template) return;
    let live = true;
    setDetail(null);
    setDetailError(null);
    api
      .scaffoldOptions(template.short_names[0], language || null)
      .then((d) => {
        if (!live) return;
        setDetail(d);
        setValues(initialValues(d.options));
      })
      .catch((e: unknown) => {
        if (live) setDetailError(e instanceof Error ? e.message : String(e));
      });
    return () => {
      live = false;
    };
  }, [template, language]);

  // The three fields that are not the template's: what the project is called,
  // where its document goes, and where its files land. The path and the
  // output follow the name until the person touches them — after that they
  // are theirs, and a later rename must not overwrite what they typed.
  const [name, setName] = useState("Greeter");
  const [output, setOutput] = useState("greeter");
  const [outputTouched, setOutputTouched] = useState(false);
  const renameTo = (next: string) => {
    setName(next);
    if (!outputTouched) setOutput(slug(next));
  };

  const [showProblems, setShowProblems] = useState(false);
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);
  /** The folder is not a repository: a screen of its own, keyed off the
   * response's `missing` field, since a scaffold is a commit and there is
   * nowhere to make one. */
  const [noRepository, setNoRepository] = useState(false);

  const spec: ScaffoldSpec | null = useMemo(() => {
    if (!template || !detail) return null;
    return {
      template: template.short_names[0],
      title: template.name,
      language: language || null,
      name,
      output,
      image: catalog.image,
      options: chosenOptions(detail.options, values),
    };
  }, [template, detail, language, name, output, catalog.image, values]);

  // The preview: the server's own renderer, debounced. Held across a refresh
  // rather than blanked, so the pane does not flicker empty on every keypress.
  const [preview, setPreview] = useState<{ command: string; message: string } | null>(
    null,
  );
  useEffect(() => {
    if (!spec) return;
    let live = true;
    const timer = setTimeout(() => {
      api
        .scaffoldPreview(spec)
        .then((p) => live && setPreview({ command: p.command, message: p.message }))
        .catch(() => {
          // A preview that cannot be rendered is not an error the person has
          // to dismiss — the button says what went wrong if they press it.
        });
    }, PREVIEW_DEBOUNCE_MS);
    return () => {
      live = false;
      clearTimeout(timer);
    };
  }, [spec]);

  const problems = validate(name, output);
  const blocked = Object.keys(problems).length > 0;

  const submit = useCallback(() => {
    if (blocked) {
      setShowProblems(true);
      return;
    }
    if (!spec || busy) return;
    setBusy(true);
    setFailure(null);
    api
      .scaffoldCreate(spec)
      .then((created) => {
        onCreated(created);
        onClose();
      })
      .catch((e: unknown) => {
        const missingRepository =
          e instanceof ApiError &&
          typeof e.body === "object" &&
          e.body !== null &&
          (e.body as { missing?: string }).missing === "repository";
        if (missingRepository) setNoRepository(true);
        else setFailure(e instanceof Error ? e.message : String(e));
        setBusy(false);
      });
  }, [blocked, spec, busy, onCreated, onClose]);

  const searchRef = useRef<HTMLInputElement>(null);
  useEffect(() => searchRef.current?.focus(), []);

  const move = (delta: number) => {
    const index = matches.findIndex((m) => m.short_names[0] === chosen);
    const next = matches[Math.min(matches.length - 1, Math.max(0, index + delta))];
    if (next) setChosen(next.short_names[0]);
  };

  if (noRepository) return <NoRepositoryScreen onClose={onClose} />;

  return (
    <Shell onClose={busy ? () => {} : onClose} label="New project">
      <div
        className="new-project__body"
        onKeyDown={(event) => {
          if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
            event.preventDefault();
            submit();
          }
        }}
      >
        <div className="insert-menu__list-pane">
          <input
            ref={searchRef}
            className="insert-menu__search"
            type="search"
            placeholder="Find a template…"
            value={query}
            aria-label="Find a template"
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "ArrowDown") {
                event.preventDefault();
                move(1);
              } else if (event.key === "ArrowUp") {
                event.preventDefault();
                move(-1);
              }
            }}
          />
          <div className="insert-menu__list" role="listbox" aria-label="Templates">
            {matches.length === 0 && (
              <p className="insert-menu__empty">
                Nothing by that name. This is whatever{" "}
                <span className="mono">dotnet new list</span> says — a template
                from a package needs{" "}
                <span className="mono">dotnet new install</span> first.
              </p>
            )}
            {grouped(matches).map(({ group, templates }) => (
              <div key={group} className="insert-menu__group">
                <h3 className="insert-menu__group-title">{group}</h3>
                {templates.map((candidate) => (
                  <button
                    key={candidate.short_names[0]}
                    type="button"
                    role="option"
                    aria-selected={candidate.short_names[0] === template?.short_names[0]}
                    className={`insert-menu__item${
                      candidate.short_names[0] === template?.short_names[0] ? " on" : ""
                    }`}
                    onClick={() => setChosen(candidate.short_names[0])}
                    onDoubleClick={submit}
                  >
                    <span className="insert-menu__item-title">{candidate.name}</span>
                    <span className="insert-menu__item-tag mono">
                      {candidate.short_names[0]}
                    </span>
                    <span className="insert-menu__item-summary">
                      {candidate.languages.join(", ") || "no language"}
                    </span>
                  </button>
                ))}
              </div>
            ))}
          </div>
          <p className="new-project__sdk">
            .NET SDK <span className="mono">{catalog.sdk_version}</span>
          </p>
        </div>

        <div className="insert-menu__form-pane">
          <div className="insert-menu__form-scroll">
            <header className="insert-menu__header">
              <h2 className="insert-menu__title">{template?.name ?? "…"}</h2>
              <code className="insert-menu__tag">
                dotnet new {template?.short_names[0]}
              </code>
            </header>
            {detail?.description && (
              <p className="insert-menu__summary">{detail.description}</p>
            )}

            <div className="insert-menu__fields">
              <Field
                label="Project name"
                attr="-n"
                hint="The .NET root namespace and the assembly name."
                problem={showProblems ? problems.name : undefined}
              >
                <input
                  className={`insert-menu__input${showProblems && problems.name ? " bad" : ""}`}
                  value={name}
                  spellCheck={false}
                  onChange={(event) => renameTo(event.target.value)}
                />
              </Field>

              <Field
                label="Files land in"
                attr="-o"
                hint="The folder the scaffold is committed into, inside the one you have open. It must be empty."
                problem={showProblems ? problems.output : undefined}
              >
                <input
                  className={`insert-menu__input${showProblems && problems.output ? " bad" : ""}`}
                  value={output}
                  spellCheck={false}
                  onChange={(event) => {
                    setOutputTouched(true);
                    setOutput(event.target.value);
                  }}
                />
              </Field>

              {template && template.languages.length > 1 && (
                <Field
                  label="Language"
                  attr="--language"
                  hint="Changing this re-reads the template's options: they differ per language."
                >
                  <select
                    className="insert-menu__input"
                    value={language}
                    onChange={(event) => setLanguage(event.target.value)}
                  >
                    {template.languages.map((l) => (
                      <option key={l} value={l}>
                        {l}
                      </option>
                    ))}
                  </select>
                </Field>
              )}

              {detailError && (
                <p className="insert-menu__problem" role="alert">
                  {detailError}
                </p>
              )}
              {!detail && !detailError && (
                <p className="insert-menu__hint">Reading this template's options…</p>
              )}

              {detail?.options.map((option) => (
                <OptionField
                  key={option.flag}
                  option={option}
                  value={values[option.flag] ?? ""}
                  onChange={(next) =>
                    setValues((current) => ({ ...current, [option.flag]: next }))
                  }
                />
              ))}
            </div>
          </div>

          <div className="insert-menu__preview">
            <h3 className="insert-menu__preview-title">The commit this makes</h3>
            <pre className="insert-menu__preview-text mono">
              {preview?.message ?? "…"}
            </pre>
            <p className="insert-menu__hint">
              The scaffolder runs and its output is committed, as one act, with
              the command in the commit's trailers. Nothing of yours is swept in
              and nothing is edited before it is committed — which is what lets
              the commit be replayed with a newer SDK later. Your changes go in
              the next commit.
            </p>
          </div>

          {failure && (
            <p className="insert-menu__problem" role="alert">
              {failure}
            </p>
          )}

          <div className="insert-menu__actions">
            <span className="insert-menu__shortcut">
              {navigator.platform.startsWith("Mac") ? "⌘" : "Ctrl"}+Enter
            </span>
            <button type="button" className="btn" onClick={onClose} disabled={busy}>
              Cancel
            </button>
            <button
              type="button"
              className="btn btn-primary"
              onClick={submit}
              disabled={busy || !detail}
              data-tip={blocked ? "Fill the fields marked below first" : undefined}
            >
              {busy ? "Scaffolding…" : "Create project"}
            </button>
          </div>
        </div>
      </div>
    </Shell>
  );
}

function Field({
  label,
  attr,
  hint,
  problem,
  children,
}: {
  label: string;
  attr: string;
  hint: string;
  problem?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="insert-menu__field">
      <label className="insert-menu__label">
        {label}
        <span className="mono insert-menu__attr">{attr}</span>
      </label>
      {children}
      <p className="insert-menu__hint">{hint}</p>
      {problem && (
        <p className="insert-menu__problem" role="alert">
          {problem}
        </p>
      )}
    </div>
  );
}

/** One of the template's own options. */
function OptionField({
  option,
  value,
  onChange,
}: {
  option: TemplateOption;
  value: string;
  onChange: (next: string) => void;
}) {
  const hint =
    option.flag === NO_RESTORE
      ? "On by default here, unlike dotnet's: a restore is a build step, and what it writes into obj/ is not something a document should own."
      : option.description;
  return (
    <div className="insert-menu__field">
      <label className="insert-menu__label">
        {option.names.join(", ")}
        {option.default !== null && (
          <span className="mono insert-menu__attr">default {option.default}</span>
        )}
      </label>
      {option.kind === "bool" ? (
        <label className="new-project__check">
          <input
            type="checkbox"
            checked={(value || "false").toLowerCase() === "true"}
            onChange={(event) => onChange(event.target.checked ? "true" : "false")}
          />
          {option.description}
        </label>
      ) : option.kind === "choice" ? (
        <select
          className="insert-menu__input"
          value={value}
          onChange={(event) => onChange(event.target.value)}
        >
          {/* A choice may be left alone: no flag is written, and `dotnet`
              decides. Not the same as picking the value that happens to be
              the default today. */}
          <option value="">(leave to dotnet)</option>
          {option.choices.map((choice) => (
            <option key={choice.value} value={choice.value}>
              {choice.description
                ? `${choice.value} — ${choice.description}`
                : choice.value}
            </option>
          ))}
        </select>
      ) : (
        <input
          className="insert-menu__input"
          value={value}
          spellCheck={false}
          placeholder={option.default ?? ""}
          onChange={(event) => onChange(event.target.value)}
        />
      )}
      {option.kind !== "bool" && <p className="insert-menu__hint">{hint}</p>}
      {option.kind === "bool" && option.flag === NO_RESTORE && (
        <p className="insert-menu__hint">{hint}</p>
      )}
      {option.enabled_if && (
        <p className="insert-menu__hint">
          Applies when <span className="mono">{option.enabled_if}</span>.
        </p>
      )}
    </div>
  );
}

/**
 * The folder is not a repository. A sentence and a next step, not a button:
 * `git init` is a decision about the folder, and the terminal is one click
 * away on the tree.
 */
function NoRepositoryScreen({ onClose }: { onClose: () => void }) {
  return (
    <Shell onClose={onClose} label="New project">
      <div className="new-project__message">
        <h2>This folder is not a git repository</h2>
        <p>
          A new project is a commit that carries the command that made it, so
          it needs a repository to be recorded in.
        </p>
        <p className="insert-menu__hint">
          Open a terminal on the folder’s row in the tree and run{" "}
          <span className="mono">git init</span>, then try again.
        </p>
        <div className="insert-menu__actions">
          <button type="button" className="btn" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </Shell>
  );
}

