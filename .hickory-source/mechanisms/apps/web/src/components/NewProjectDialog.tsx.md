<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/NewProjectDialog.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-65e48889b856"></a>
## `apps/web/src/components/NewProjectDialog.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A New Project Is A Recipe Commit, Made As One Act](../../../../../guarantees/authoring/a-new-project-is-a-recipe-commit/contract.md), [New Project Offers The Scaffolder's Own Options, Not A List This Product Maintains](../../../../../guarantees/authoring/a-new-project-reads-the-scaffolder-s-own-options/contract.md), [A New Project Uses The Toolchain The Language Uses](../../../../../guarantees/authoring/a-new-project-uses-the-toolchain-the-language-uses/contract.md), [A write exposes its proposed effects](../../../../../guarantees/authoring/a-write-exposes-its-proposed-effects/contract.md), [Install a supported missing debugger and resume the requested launch](../../../../../guarantees/debugging/a-missing-debugger-is-a-button/contract.md), [A Command The App Runs On Your Behalf Is Watched In A Terminal](../../../../../guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal/contract.md), [A failed action keeps its editor](../../../../../guarantees/execution/a-failed-action-keeps-its-editor/contract.md).

<a id="f-65e48889b856-NewProjectDialogProps"></a>
### `NewProjectDialogProps`

<storycode:copy id="f-65e48889b856-NewProjectDialogProps">
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import { Field, OptionField } from "./NewProjectFields";
import { useNewProjectActions, type ProjectField } from "./useNewProjectActions";
import { ChangePreview } from "./ChangePreview";
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
// scaffolder, and commit what it wrote as one act, so the forty files it
// produced are a commit carrying the command that made them.
// docs/specs/freeform/lenses.md, step 3.
//
// Two things this dialog does NOT do, and both are deliberate.
//
// It does not report the scaffolder. Pressing the button hands you a terminal
// running `dotnet new`, and the dialog gets out of the way: a command a person
// asked for is watched, never summarised, because a failing command says why
// in its own words and "Unprocessable Entity" says nothing at all.
// docs/guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal.md
//
// And it does not confine you to the folder the app has open. The location is
// anywhere on this machine; the repository that records the recipe is
// whichever one holds it, which the server resolves and the preview names.
//
// Which is why the last two fields are checkboxes rather than screens. A
// location inside no repository was, briefly, a screen with a `git init`
// button on it — better than the sentence it replaced, and still a stop sign
// in the middle of an act somebody had already committed to. The same is true
// of opening the project: these are decisions about what pressing the button
// does, so they belong beside the button, ticked correctly before you get
// there and yours to untick. A form that has already made the obvious choice
// is faster than one that asks, and honest as long as the choice is visible.

import { useCallback, useEffect, useId, useMemo, useRef, useState } from "react";

import type { UiOutcome } from "../lib/uiActions";
import { ApiError, api } from "../api/client";
import type {
  ScaffoldCatalog,
  ScaffoldPreview,
  ScaffoldStarted,
  ScaffoldSpec,
  ScaffoldTemplate,
  ScaffoldTemplateDetail,
} from "../api/types";
import {
  chosenOptions,
  filterTemplates,
  grouped,
  initialLanguage,
  initialValues,
  joinPath,
  problems as validate,
  slug,
  type FieldValues,
} from "../lib/scaffold";

export interface NewProjectDialogProps {
  /** The scaffolder is running, in the terminal named here. The commit comes
   * later, when it exits; whoever opened this dialog watches for that, since
   * the dialog is gone by then. */
  onStarted: (started: ScaffoldStarted) => void;
  onClose: () => void;
}

</storycode:copy>
<a id="f-65e48889b856-isInside"></a>
### `isInside`

Whether the project lands inside the folder this window already shows.
The one thing "open the project" has to know. A project made beside your
notes is already on screen — the tree will show it the moment it is
committed — so opening a window on it is noise. A project made in
`~/src/thing` is somewhere this window cannot see, and not opening it
leaves you with a commit you have to go and find.

<storycode:copy id="f-65e48889b856-isInside">
/**
 * Whether the project lands inside the folder this window already shows.
 *
 * The one thing "open the project" has to know. A project made beside your
 * notes is already on screen — the tree will show it the moment it is
 * committed — so opening a window on it is noise. A project made in
 * `~/src/thing` is somewhere this window cannot see, and not opening it
 * leaves you with a commit you have to go and find.
 */
function isInside(folder: string | null, open: string): boolean {
  if (!folder) return false;
  const root = open.replace(/[/\\]+$/, "");
  return folder === root || folder.startsWith(`${root}/`) || folder.startsWith(`${root}\\`);
}

</storycode:copy>
<a id="f-65e48889b856-PREVIEW_DEBOUNCE_MS"></a>
### `PREVIEW_DEBOUNCE_MS`

How long to sit still before asking the server what the commit looks
like. Long enough that typing a name is one request rather than eight,
short enough that the preview feels like it is following you.

<storycode:copy id="f-65e48889b856-PREVIEW_DEBOUNCE_MS">
/** How long to sit still before asking the server what the commit looks
 * like. Long enough that typing a name is one request rather than eight,
 * short enough that the preview feels like it is following you. */
const PREVIEW_DEBOUNCE_MS = 180;

</storycode:copy>
<a id="f-65e48889b856-NewProjectDialog"></a>
### `NewProjectDialog`

<storycode:copy id="f-65e48889b856-NewProjectDialog">
export function NewProjectDialog({ onStarted, onClose }: NewProjectDialogProps) {
  const [catalog, setCatalog] = useState<ScaffoldCatalog | null>(null);
  /** Which scaffolder the dialog is asking about. `null` until the first
   * catalogue answers, because what this machine HAS is measured rather than
   * assumed — a picker offering `uv` where there is none is a form that
   * fails after it has been filled in. */
  const [toolchain, setToolchain] = useState<string | null>(null);
  /** Set when the chosen scaffolder is not on this machine — a different
   * screen, not an error line. Keyed off the response's `missing` field
   * rather than its wording, so a reworded sentence cannot take the screen
   * away. Holds the program's name, because the screen has to say which. */
  const [missingTool, setMissingTool] = useState<string | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [reload, setReload] = useState(0);
  const liveToolchain = useRef(toolchain); liveToolchain.current = toolchain;
  const selectToolchain = (value: string) => {
    if (value === toolchain || !catalog?.toolchains.some(tool => tool.id === value)) return;
    setCatalog(null); setToolchain(value);
  };
  const statusParent = `project-catalog:${useId()}`;
  const statusRevision = useUiRevision(() => [toolchain, catalog?.kind, missingTool, loadError, reload]);
  const invokeCatalog = useUiCommands(loadError ? [{ key: statusParent, command: {
    parent: statusParent, subject: statusParent, label: "Retry new project template catalog", parameters: {}, prerequisites: [],
    read: () => ({ revision: statusRevision(), enabled: !!loadError }),
    execute: async () => { setReload(current => current + 1); return { status: "completed", value: { retry_requested: true } }; },
  } }] : []);
  useUiReading("new project catalog status", { toolchain, loading: !catalog && !missingTool && !loadError,
    missing_tool: missingTool, error: loadError, catalog_kind: catalog?.kind ?? null });

  useEffect(() => {
    let live = true;
    setMissingTool(null);
    setLoadError(null);
    api
      .scaffoldTemplates(toolchain ?? undefined)
      .then((c) => {
        if (!live || liveToolchain.current !== toolchain) return;
        setCatalog(c);
        setToolchain((current) => current ?? c.kind);
      })
      .catch((e: unknown) => {
        if (!live || liveToolchain.current !== toolchain) return;
        const missing =
          e instanceof ApiError &&
          typeof e.body === "object" &&
          e.body !== null
            ? (e.body as { missing?: string }).missing
            : undefined;
        if (missing) setMissingTool(missing);
        else setLoadError(e instanceof Error ? e.message : String(e));
      });
    return () => {
      live = false;
    };
  }, [toolchain, reload]);

  if (missingTool === "dotnet") return <NoSdkScreen onClose={onClose} />;
  if (missingTool) return <NoToolScreen tool={missingTool} onClose={onClose} />;
  if (loadError)
    return (
      <Shell onClose={onClose} label="New project">
        <div className="new-project__message">
          <h2>Could not read the templates</h2>
          <p className="new-project__error">{loadError}</p>
          <p className="insert-menu__hint">Your project has not been created. Retry after addressing the reported problem.</p>
          <div className="insert-menu__actions">
            <button type="button" className="btn" onClick={() => void invokeCatalog(statusParent, {}).catch(() => {})}>Retry reading templates</button>
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

  return (
    // Keyed on the scaffolder: choosing another one is a different form with
    // a different template, different fields and a different name, and
    // carrying `--no-restore` from `dotnet new` into `uv init` would be a
    // flag uv has never heard of.
    <Chooser
      key={toolchain ?? catalog.kind}
      catalog={catalog}
      toolchain={toolchain ?? catalog.kind}
      onToolchain={selectToolchain}
      onStarted={onStarted}
      onClose={onClose}
    />
  );
}

</storycode:copy>
<a id="f-65e48889b856-Shell"></a>
### `Shell`

The backdrop and the panel, shared by every state this dialog has.

<storycode:copy id="f-65e48889b856-Shell">
/** The backdrop and the panel, shared by every state this dialog has. */
function Shell({
  children,
  onClose,
  label,
  canClose = () => true,
}: {
  children: React.ReactNode;
  onClose: () => void;
  label: string;
  canClose?: () => boolean;
}) {
  const parent = `project-dialog:${useId()}`;
  const revision = useUiRevision(() => [label, canClose()]);
  const close = () => { if (canClose()) onClose(); };
  const invoke = useUiCommands([{ key: parent, command: { parent, subject: parent, label: "Close new project dialog", parameters: {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: canClose(), reason: canClose() ? undefined : "Wait for the scaffold start reply" }),
    execute: async () => { if (!canClose()) return { status: "unavailable", reason: "Wait for the scaffold start reply" }; close(); return { status: "completed", value: { close_requested: true } }; },
  } }]);
  const requestClose = () => void invoke(parent, {}).catch(() => {});
  return (
    <div className="insert-menu-backdrop" onMouseDown={requestClose}>
      <div
        className="new-project"
        role="dialog"
        aria-label={label}
        onMouseDown={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          if (event.key === "Escape") {
            event.preventDefault();
            requestClose();
          }
        }}
      >
        {children}
      </div>
    </div>
  );
}

</storycode:copy>
<a id="f-65e48889b856-NoSdkScreen"></a>
### `NoSdkScreen`

No SDK on this machine.
A sentence and a link, not a button: unlike a debug adapter, the .NET SDK
is not something this product has a catalogue for or any business fetching
— it is a several-hundred-megabyte platform install with its own installer
and its own licence. Offering a button we cannot honour would be worse than
saying plainly where to get it.

<storycode:copy id="f-65e48889b856-NoSdkScreen">
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
          <a href="https://dotnet.microsoft.com/download" target="_blank" rel="noreferrer">the .NET SDK download page</a>,
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

</storycode:copy>
<a id="f-65e48889b856-NoToolScreen"></a>
### `NoToolScreen`

The same screen as `NoSdkScreen`, for a scaffolder that is not `dotnet`.
Kept apart rather than generalised because .NET's is the one with a
download page worth naming, and because this product installs language
servers and debug adapters but never a language toolchain — that is a
sentence and not a button, for the reason `owning-what-a-scaffolder-wrote`
gives about the SDK.

<storycode:copy id="f-65e48889b856-NoToolScreen">
/**
 * The same screen as `NoSdkScreen`, for a scaffolder that is not `dotnet`.
 *
 * Kept apart rather than generalised because .NET's is the one with a
 * download page worth naming, and because this product installs language
 * servers and debug adapters but never a language toolchain — that is a
 * sentence and not a button, for the reason `owning-what-a-scaffolder-wrote`
 * gives about the SDK.
 */
function NoToolScreen({ tool, onClose }: { tool: string; onClose: () => void }) {
  return (
    <Shell onClose={onClose} label="New project">
      <div className="new-project__message">
        <h2>No {tool} on this machine</h2>
        <p>
          New Project asks <code>{tool}</code> what it can scaffold, and there
          is no <code>{tool}</code> on this machine's PATH.
        </p>
        <p className="insert-menu__hint">
          Install it, then reopen this window — a running process keeps the
          PATH it was started with, so a tool installed just now will not be
          visible until it restarts.
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

</storycode:copy>
<a id="f-65e48889b856-ToolchainPicker"></a>
### `ToolchainPicker`

Which scaffolder to use.
Drawn only when this machine has more than one: a picker with a single
entry is a control that cannot be operated, and on a .NET-only machine
this dialog should look exactly as it did.

<storycode:copy id="f-65e48889b856-ToolchainPicker">
/**
 * Which scaffolder to use.
 *
 * Drawn only when this machine has more than one: a picker with a single
 * entry is a control that cannot be operated, and on a .NET-only machine
 * this dialog should look exactly as it did.
 */
function ToolchainPicker({
  catalog,
  chosen,
  onChoose,
}: {
  catalog: ScaffoldCatalog;
  chosen: string;
  onChoose: (id: string) => void;
}) {
  if (catalog.toolchains.length < 2) return null;
  return (
    <div className="new-project__toolchains" role="radiogroup" aria-label="Toolchain">
      {catalog.toolchains.map((tool) => (
        <button
          key={tool.id}
          type="button"
          role="radio"
          aria-checked={tool.id === chosen}
          className={`new-project__toolchain${tool.id === chosen ? " on" : ""}`}
          onClick={() => onChoose(tool.id)}
          data-tip={`${tool.label} — ${tool.version}`}
        >
          {tool.label}
        </button>
      ))}
    </div>
  );
}

</storycode:copy>
<a id="f-65e48889b856-Chooser"></a>
### `Chooser`

<storycode:copy id="f-65e48889b856-Chooser">
function Chooser({
  catalog,
  toolchain,
  onToolchain,
  onStarted,
  onClose,
}: {
  toolchain: string;
  onToolchain: (id: string) => void;
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
  const [capturedDetail, setDetail] = useState<{ value: ScaffoldTemplateDetail; key: string } | null>(null);
  const detailKey = JSON.stringify([template?.short_names[0], language, toolchain]);
  const liveDetailKey = useRef(detailKey); liveDetailKey.current = detailKey;
  const detail = capturedDetail?.key === detailKey ? capturedDetail.value : null;
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
      .scaffoldOptions(template.short_names[0], language || null, toolchain)
      .then((d) => {
        if (!live || liveDetailKey.current !== detailKey) return;
        setDetail({ value: d, key: detailKey });
        setValues(initialValues(d.options));
      })
      .catch((e: unknown) => {
        if (live && liveDetailKey.current === detailKey) setDetailError(e instanceof Error ? e.message : String(e));
      });
    return () => {
      live = false;
    };
  }, [template, language, toolchain]);

  // The three fields that are not the template's: what the project is called,
  // where it is made, and what its folder is called. The folder follows the
  // name until the person touches it — after that it is theirs, and a later
  // rename must not overwrite what they typed. The location starts on the
  // folder the app has open and is not tied to it: a project may be made
  // anywhere on this machine.
  const [name, setName] = useState("Greeter");
  const [location, setLocation] = useState(catalog.location);
  const [output, setOutput] = useState("greeter");
  const [outputTouched, setOutputTouched] = useState(false);
  const renameTo = (next: string) => {
    if (pending.current || pickingRef.current) return;
    setName(next);
    if (!outputTouched) setOutput(slug(next));
  };

  // The two checkboxes, and the reason both track the location.
  //
  // Each one has an answer that is right nearly always, and the right answer
  // changes as you type: a location outside a repository needs one made, and
  // a project made somewhere new wants opening while one made inside the
  // folder you are already looking at does not. So each starts derived and
  // stops the moment the person touches it — the same rule the folder name
  // follows, for the same reason: a later keystroke must never quietly
  // overwrite a decision somebody made on purpose.
  const [initRepository, setInitRepository] = useState(false);
  const [initTouched, setInitTouched] = useState(false);
  const [openIt, setOpenIt] = useState(false);
  const [openTouched, setOpenTouched] = useState(false);
  const [newWindow, setNewWindow] = useState(true);

  /** A native modal is up. The button is disabled while it is, because the
   * dialog is blocking on the other side and a second one cannot be shown. */
  const [picking, setPicking] = useState(false);
  const [showProblems, setShowProblems] = useState(false);
  const [busy, setBusy] = useState(false);
  const pending = useRef(false);
  const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const [failure, setFailure] = useState<string | null>(null);

  const spec: ScaffoldSpec | null = useMemo(() => {
    if (!template || !detail) return null;
    return {
      template: template.short_names[0],
      title: template.name,
      language: language || null,
      name,
      output,
      location,
      init_repository: initRepository,
      open: openIt ? (newWindow ? "new-window" : "this-window") : "none",
      image: catalog.image,
      options: chosenOptions(detail.options, values),
      toolchain,
    };
  }, [
    toolchain,
    template,
    detail,
    language,
    name,
    output,
    location,
    initRepository,
    openIt,
    newWindow,
    catalog.image,
    values,
  ]);

  // The preview: the server's own renderer, debounced. Held across a refresh
  // rather than blanked, so the pane does not flicker empty on every keypress.
  //
  // It also answers the question the location field raises — *which
  // repository will record this?* — and answers it while the person is still
  // typing, including "none of them, and here is the folder a `git init`
  // would run in". That is why the preview route never refuses for want of a
  // repository: a blank pane teaches nothing.
  const specKey = JSON.stringify(spec);
  const liveSpec = useRef(specKey); liveSpec.current = specKey;
  const [previewKey, setPreviewKey] = useState<string | null>(null);
  const [preview, setPreview] = useState<ScaffoldPreview | null>(null);
  useEffect(() => {
    if (!spec) return;
    let live = true;
    const timer = setTimeout(() => {
      api
        .scaffoldPreview(spec)
        .then((p) => {
          if (!live || liveSpec.current !== specKey) return;
          setPreview(p); setPreviewKey(specKey);
          // The preview is what knows both answers, because both are about
          // where the project lands and only the server resolves that.
          if (!initTouched) setInitRepository(p.repository === null);
          if (!openTouched) setOpenIt(!isInside(p.folder, catalog.location));
        })
        .catch(() => {
          // A preview that cannot be rendered is not an error the person has
          // to dismiss — the button says what went wrong if they press it.
        });
    }, PREVIEW_DEBOUNCE_MS);
    return () => {
      live = false;
      clearTimeout(timer);
    };
  }, [spec, initTouched, openTouched, catalog.location]);

  const problems = validate(name, output);
  const blocked = Object.keys(problems).length > 0;

  const submit = useCallback(async (signal?: AbortSignal): Promise<UiOutcome> => {
    if (blocked) { setShowProblems(true); return { status: "unavailable", reason: "Correct the marked project fields" }; }
    if (!spec || pending.current || signal?.aborted || !mounted.current || liveSpec.current !== JSON.stringify(spec))
      return { status: "unavailable", reason: "Wait for the current project options or pending request" };
    pending.current = true; setBusy(true); setFailure(null);
    try {
      const started = await api.scaffoldCreate(spec);
      if (mounted.current) { onStarted(started); onClose(); }
      return { status: "running", job: started.session.id, value: { started: JSON.parse(JSON.stringify(started)),
        committed: false, scope: "Actual scaffold terminal; inspect its result and output, not a completed commit" } };
    } catch (e) {
      const reason = e instanceof Error ? e.message : String(e);
      if (mounted.current) setFailure(reason);
      return { status: "unavailable", reason };
    } finally { pending.current = false; if (mounted.current) setBusy(false); }
  }, [blocked, spec, onStarted, onClose]);
  const chooseOutput = (value: string) => { if (pending.current || pickingRef.current) return; setOutputTouched(true); setOutput(value); };
  const chooseInit = (value: boolean) => { if (pending.current || pickingRef.current) return; setInitTouched(true); setInitRepository(value); };
  const chooseOpen = (value: boolean) => { if (pending.current || pickingRef.current) return; setOpenTouched(true); setOpenIt(value); };
  const pickingRef = useRef(false);
  const pick = async (): Promise<UiOutcome> => {
    if (!catalog.can_pick_folder || pickingRef.current || pending.current) return { status: "unavailable", reason: "Native folder picker is unavailable or busy; set the location field directly" };
    pickingRef.current = true; setPicking(true);
    const captured = location;
    try {
      const chosen = await api.pickFolder(captured);
      if (!mounted.current) return { status: "unavailable", reason: "Project dialog closed during folder selection" };
      if (chosen.path) setLocation(current => current === captured ? chosen.path! : current);
      return { status: "completed", value: { selected_path: chosen.path, cancelled: !chosen.path,
        scope: "Native chooser response; typed location is the direct equivalent" } };
    } catch (error) {
      const reason = error instanceof Error ? error.message : String(error); if (mounted.current) setFailure(reason);
      return { status: "unavailable", reason };
    } finally { pickingRef.current = false; if (mounted.current) setPicking(false); }
  };
  const text = { type: "string" as const, maxLength: 8192 };
  const boolean = { type: "boolean" as const };
  const fields: ProjectField[] = [
    { id: "query", label: "Search new project templates", schema: text, value: query, set: value => setQuery(value as string) },
    { id: "name", label: "Set new project name", schema: text, value: name, set: value => renameTo(value as string) },
    { id: "location", label: "Set new project location", schema: text, value: location, set: value => setLocation(value as string) },
    { id: "output", label: "Set new project folder name", schema: text, value: output, set: value => chooseOutput(value as string) },
    { id: "init", label: "Set new project repository creation", schema: boolean, value: initRepository, enabled: !preview?.repository, set: value => chooseInit(value as boolean) },
    { id: "open", label: "Set new project opening", schema: boolean, value: openIt, set: value => chooseOpen(value as boolean) },
    { id: "window", label: "Set new project separate window", schema: boolean, value: newWindow, enabled: openIt, set: value => setNewWindow(value as boolean) },
  ];
  if (template && template.languages.length > 1) fields.push({ id: "language", label: "Set new project language", schema: { ...text, choices: template.languages }, value: language, set: value => setLanguage(value as string) });
  if (catalog.toolchains.length > 1) fields.push({ id: "toolchain", label: "Set new project toolchain", schema: { ...text, choices: catalog.toolchains.map(tool => tool.id) }, value: toolchain, set: value => onToolchain(value as string) });
  for (const option of detail?.options ?? []) fields.push({ id: `option:${option.flag}`, label: `Set new project option ${option.flag}`,
    schema: option.kind === "bool" ? { ...text, choices: ["true", "false"] } : option.kind === "choice" ? { ...text, choices: ["", ...option.choices.map(choice => choice.value)] } : text,
    value: values[option.flag] ?? "", set: value => setValues(current => ({ ...current, [option.flag]: value as string })) });
  const invokeProject = useNewProjectActions({ toolchain, templates: matches, template, language, detail, detail_error: detailError,
    name, location, output, init_repository: initRepository, open: openIt, new_window: newWindow, values, spec,
    preview, preview_current: previewKey === specKey, problems, busy, picking, failure },
    () => mounted.current && !pending.current && !pickingRef.current, fields, [
      ...matches.map(candidate => ({ id: `template:${candidate.short_names[0]}`, label: `Choose new project template ${candidate.name}`,
        enabled: () => true, execute: async () => { setChosen(candidate.short_names[0]); return { status: "completed" as const, value: { template_selected: candidate.short_names[0] } }; } })),
      { id: "create", label: "Create new project", enabled: () => !!detail, execute: async (_args, _actor, signal) => submit(signal) },
      { id: "pick", label: "Choose new project folder", enabled: () => catalog.can_pick_folder && !pickingRef.current, reason: () => "This build has no available native folder picker; set the location field directly", execute: pick },
    ]);

  const searchRef = useRef<HTMLInputElement>(null);
  useEffect(() => searchRef.current?.focus(), []);

  const move = (delta: number) => {
    if (pending.current || pickingRef.current) return;
    const index = matches.findIndex((m) => m.short_names[0] === chosen);
    const next = matches[Math.min(matches.length - 1, Math.max(0, index + delta))];
    if (next) setChosen(next.short_names[0]);
  };

  return (
    <Shell onClose={onClose} canClose={() => !pending.current} label="New project">
      <div
        className="new-project__body"
        onKeyDown={(event) => {
          if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
            event.preventDefault();
            invokeProject("create");
          }
        }}
      >
        <div className="insert-menu__list-pane">
          <ToolchainPicker
            catalog={catalog}
            chosen={toolchain}
            onChoose={value => { if (!pending.current && !pickingRef.current) onToolchain(value); }}
          />
          <input
            ref={searchRef}
            className="insert-menu__search"
            type="search"
            placeholder="Find a template…"
            hidden={catalog.templates.length < 2}
            value={query}
            aria-label="Find a template"
            onChange={(event) => { if (!pending.current && !pickingRef.current) setQuery(event.target.value); }}
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
                <span className="mono">
                  {toolchain === "dotnet" ? "dotnet new list" : `${toolchain} --help`}
                </span>{" "}
                says.
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
                    onClick={() => { if (!pending.current && !pickingRef.current) setChosen(candidate.short_names[0]); }}
                    onDoubleClick={() => invokeProject("create")}
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
            {toolchain === "dotnet" ? ".NET SDK " : ""}
            <span className="mono">{catalog.sdk_version}</span>
          </p>
        </div>

        <div className="insert-menu__form-pane">
          <div className="insert-menu__form-scroll">
            <header className="insert-menu__header">
              <h2 className="insert-menu__title">{template?.name ?? "…"}</h2>
              <code className="insert-menu__tag">
                {/* The tool's own phrase. A single-shape scaffolder has no
                    template to name, and `uv init init` reads as a mistake. */}
                {catalog.toolchains.find((t) => t.id === toolchain)?.templated ?? true
                  ? `${toolchain === "dotnet" ? "dotnet new" : toolchain} ${template?.short_names[0] ?? ""}`
                  : `${toolchain} ${template?.short_names[0] ?? ""}`}
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
                label="Location"
                attr="anywhere"
                hint="The folder the project's own folder is made in. Any folder on this machine — the repository that records it is whichever one holds it, and `~` and a relative path both work."
              >
                <div className="new-project__path">
                  <input
                    className="insert-menu__input"
                    value={location}
                    spellCheck={false}
                    autoComplete="off"
                    aria-label="Location"
                    onChange={(event) => { if (!pending.current && !pickingRef.current) setLocation(event.target.value); }}
                  />
                  {/* Drawn only where there is something on the other end of
                      it: a browser tab served by `hick up` has no native
                      dialogs, and a button that always fails is worse than a
                      field you type into. */}
                  {catalog.can_pick_folder && (
                    <button
                      type="button"
                      className="btn new-project__browse"
                      aria-label="Choose a folder"
                      data-tip="Choose a folder"
                      disabled={picking}
                      onClick={() => invokeProject("pick")}
                    >
                      …
                    </button>
                  )}
                </div>
              </Field>

              <label className="new-project__check">
                <input
                  type="checkbox"
                  checked={initRepository}
                  disabled={Boolean(preview?.repository)}
                  onChange={(event) => {
                    chooseInit(event.target.checked);
                  }}
                />
                <span>
                  Create a git repository
                  {preview?.repository ? (
                    <span className="insert-menu__hint">
                      Already in one:{" "}
                      <span className="mono">{preview.repository}</span>
                    </span>
                  ) : (
                    <span className="insert-menu__hint">
                      A new project is a commit that carries the command that
                      made it, so it needs a repository to be recorded in.
                      Without this, a location outside one is refused.
                    </span>
                  )}
                </span>
              </label>

              <Field
                label="Folder name"
                attr="-o"
                hint="The project's own folder, made inside the location. It must not already hold anything — a scaffold is committed exactly as the scaffolder wrote it."
                problem={showProblems ? problems.output : undefined}
              >
                <input
                  className={`insert-menu__input${showProblems && problems.output ? " bad" : ""}`}
                  value={output}
                  spellCheck={false}
                  onChange={(event) => {
                    chooseOutput(event.target.value);
                  }}
                />
                <p className="insert-menu__hint mono new-project__resolved">
                  {joinPath(location, output, catalog.separator)}
                </p>
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
                    onChange={(event) => { if (!pending.current && !pickingRef.current) setLanguage(event.target.value); }}
                  >
                    {template.languages.map((l) => (
                      <option key={l} value={l}>
                        {l}
                      </option>
                    ))}
                  </select>
                </Field>
              )}

              <label className="new-project__check">
                <input
                  type="checkbox"
                  checked={openIt}
                  onChange={(event) => {
                    chooseOpen(event.target.checked);
                  }}
                />
                <span>
                  Open the project when it is made
                  <span className="insert-menu__hint">
                    Ticked on its own for a project made outside this folder,
                    since nothing here would show it otherwise.
                  </span>
                </span>
              </label>
              <label className="new-project__check new-project__check--nested">
                <input
                  type="checkbox"
                  checked={newWindow}
                  disabled={!openIt}
                  onChange={(event) => { if (!pending.current && !pickingRef.current) setNewWindow(event.target.checked); }}
                />
                <span>
                  in a new window
                  <span className="insert-menu__hint">
                    {newWindow
                      ? "A second copy of the app, on the new project. This window is left exactly as it is."
                      : "This window switches to the new project. A session is a process here, so that is a restart — every tab in this window goes, including the terminal the scaffolder just ran in."}
                  </span>
                </span>
              </label>

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
                  onChange={(next) => {
                    if (!pending.current && !pickingRef.current) setValues(current => ({ ...current, [option.flag]: next }));
                  }}
                />
              ))}
            </div>
          </div>

          <ChangePreview heading="The commit this makes">
            {preview && previewKey !== specKey && <p role="status">Preview from earlier form values; current preview is pending.</p>}
            <pre className="insert-menu__preview-text mono">
              {preview?.message ?? "…"}
            </pre>
            {preview?.repository ? (
              <p className="insert-menu__hint">
                Committed in{" "}
                <span className="mono">{preview.repository}</span>
                {preview.repository !== catalog.location &&
                  " — not the folder you have open."}
              </p>
            ) : preview?.needs_repository ? (
              <p className="insert-menu__hint">
                {initRepository ? (
                  <>
                    A git repository will be made in{" "}
                    <span className="mono">{preview.needs_repository}</span>{" "}
                    first — that is the checkbox above.
                  </>
                ) : (
                  <>
                    <span className="mono">{preview.needs_repository}</span> is
                    not inside a git repository, and nothing above says to make
                    one. This will be refused.
                  </>
                )}
              </p>
            ) : null}
            <p className="insert-menu__hint">
              The scaffolder runs in a terminal you can read, and what it wrote
              is committed the moment it exits — as one act, with the command in
              the commit's trailers. Nothing of yours is swept in and nothing is
              edited before it is committed, which is what lets the commit be
              replayed with a newer SDK later. Your changes go in the next
              commit.
            </p>
          </ChangePreview>

          {failure && (
            <p className="insert-menu__problem" role="alert">
              {failure}
            </p>
          )}

          <div className="insert-menu__actions">
            <span className="insert-menu__shortcut">
              {navigator.platform.startsWith("Mac") ? "⌘" : "Ctrl"}+Enter
            </span>
            <button type="button" className="btn" onClick={() => { if (!pending.current) onClose(); }} disabled={busy}>
              Cancel
            </button>
            <button
              type="button"
              className="btn btn-primary"
              onClick={() => invokeProject("create")}
              disabled={busy || !detail}
              data-tip={blocked ? "Fill the fields marked below first" : undefined}
            >
              {busy ? "Starting…" : "Create project"}
            </button>
          </div>
        </div>
      </div>
    </Shell>
  );
}

</storycode:copy>

### Assemble `apps/web/src/components/NewProjectDialog.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/NewProjectDialog.tsx"><storycode:paste select="#f-65e48889b856-NewProjectDialogProps"/><storycode:paste select="#f-65e48889b856-isInside"/><storycode:paste select="#f-65e48889b856-PREVIEW_DEBOUNCE_MS"/><storycode:paste select="#f-65e48889b856-NewProjectDialog"/><storycode:paste select="#f-65e48889b856-Shell"/><storycode:paste select="#f-65e48889b856-NoSdkScreen"/><storycode:paste select="#f-65e48889b856-NoToolScreen"/><storycode:paste select="#f-65e48889b856-ToolchainPicker"/><storycode:paste select="#f-65e48889b856-Chooser"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-065ce188ea68"></a>
## `apps/web/src/components/NewProjectDialog.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A New Project Is A Recipe Commit, Made As One Act](../../../../../guarantees/authoring/a-new-project-is-a-recipe-commit/contract.md), [New Project Offers The Scaffolder's Own Options, Not A List This Product Maintains](../../../../../guarantees/authoring/a-new-project-reads-the-scaffolder-s-own-options/contract.md), [A New Project Uses The Toolchain The Language Uses](../../../../../guarantees/authoring/a-new-project-uses-the-toolchain-the-language-uses/contract.md), [A write exposes its proposed effects](../../../../../guarantees/authoring/a-write-exposes-its-proposed-effects/contract.md), [A Command The App Runs On Your Behalf Is Watched In A Terminal](../../../../../guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal/contract.md), [A failed action keeps its editor](../../../../../guarantees/execution/a-failed-action-keeps-its-editor/contract.md).

<a id="f-065ce188ea68-expression_statement"></a>
### `expression_statement`

@vitest-environment jsdom
Protects
and docs/guarantees/authoring/a-machine-with-no-sdk-says-so.md

<storycode:copy id="f-065ce188ea68-expression_statement">
// @vitest-environment jsdom
//
// Protects
// docs/guarantees/authoring/a-new-project-reads-the-scaffolder-s-own-options.md
// and docs/guarantees/authoring/a-machine-with-no-sdk-says-so.md

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";

vi.mock("../api/client", () => ({
  ApiError: class ApiError extends Error {
    constructor(
      public status: number,
      message: string,
      public body?: unknown,
    ) {
      super(message);
    }
  },
  api: {
    scaffoldTemplates: vi.fn(),
    scaffoldOptions: vi.fn(),
    scaffoldPreview: vi.fn(),
    scaffoldCreate: vi.fn(),
    pickFolder: vi.fn(),
  },
}));

</storycode:copy>
<a id="f-065ce188ea68-import_statement"></a>
### `import_statement`

<storycode:copy id="f-065ce188ea68-import_statement">
import { NewProjectDialog } from "./NewProjectDialog";
</storycode:copy>
<a id="f-065ce188ea68-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-065ce188ea68-import_statement-2">
import { ApiError, api } from "../api/client";
</storycode:copy>
<a id="f-065ce188ea68-import_statement-3"></a>
### `import_statement`

<storycode:copy id="f-065ce188ea68-import_statement-3">
import type { ScaffoldCatalog, ScaffoldTemplateDetail } from "../api/types";

</storycode:copy>
<a id="f-065ce188ea68-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-065ce188ea68-expression_statement-2">
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

</storycode:copy>
<a id="f-065ce188ea68-CATALOG"></a>
### `CATALOG`

<storycode:copy id="f-065ce188ea68-CATALOG">
const CATALOG: ScaffoldCatalog = {
  kind: "dotnet",
  sdk_version: "10.0.111",
  image: "mcr.microsoft.com/dotnet/sdk:10.0",
  location: "/home/nate/notes",
  separator: "/",
  can_pick_folder: true,
  toolchains: [
    {
      id: "dotnet",
      label: ".NET",
      language: "C#",
      version: "10.0.100",
      templated: true,
    },
  ],
  templates: [
    {
      short_names: ["console"],
      name: "Console App",
      languages: ["C#", "F#"],
      default_language: "C#",
      tags: ["Common", "Console"],
    },
    {
      short_names: ["webapi"],
      name: "ASP.NET Core Web API",
      languages: ["C#"],
      default_language: "C#",
      tags: ["Web", "Web API"],
    },
  ],
};

</storycode:copy>
<a id="f-065ce188ea68-DETAIL"></a>
### `DETAIL`

<storycode:copy id="f-065ce188ea68-DETAIL">
const DETAIL: ScaffoldTemplateDetail = {
  title: "Console App (C#)",
  author: "Microsoft",
  description: "A project for creating a command-line application.",
  options: [
    {
      names: ["-f", "--framework"],
      flag: "--framework",
      kind: "choice",
      choices: [
        { value: "net10.0", description: "Target net10.0" },
        { value: "net9.0", description: "Target net9.0" },
      ],
      default: "net10.0",
      description: "The target framework for the project.",
      enabled_if: null,
    },
    {
      names: ["--no-restore"],
      flag: "--no-restore",
      kind: "bool",
      choices: [],
      default: "false",
      description: "If specified, skips the automatic restore of the project on create.",
      enabled_if: null,
    },
  ],
  other_languages: ["F#", "VB"],
};

</storycode:copy>
<a id="f-065ce188ea68-ready"></a>
### `ready`

<storycode:copy id="f-065ce188ea68-ready">
function ready() {
  vi.mocked(api.scaffoldTemplates).mockResolvedValue(CATALOG);
  vi.mocked(api.scaffoldOptions).mockResolvedValue(DETAIL);
  vi.mocked(api.scaffoldPreview).mockResolvedValue({
    output: "greeter",
    folder: "/home/nate/notes/greeter",
    repository: "/home/nate/notes",
    command: "dotnet new console -o greeter -n Greeter --language 'C#' --no-restore",
    message:
      "Scaffold Greeter with `dotnet new console`\n\nHick-Recipe: dotnet new console -o greeter -n Greeter --language 'C#' --no-restore\n",
  });
  const onStarted = vi.fn();
  const onClose = vi.fn();
  render(<NewProjectDialog onStarted={onStarted} onClose={onClose} />);
  return { onStarted, onClose };
}

</storycode:copy>
<a id="f-065ce188ea68-STARTED"></a>
### `STARTED`

What `POST /api/scaffold` answers: a terminal, not a commit.

<storycode:copy id="f-065ce188ea68-STARTED">
/** What `POST /api/scaffold` answers: a terminal, not a commit. */
const STARTED = {
  session: {
    id: "term-3",
    title: "New project: Greeter",
    cwd: "/tmp/scratch",
    monitor: false,
    state: "working",
    since_ms: 0,
    branch: null,
    dirty: false,
    preview: "",
    prompt: null,
    exit_code: null,
  },
  output: "greeter",
  folder: "/home/nate/notes/greeter",
  repository: "/home/nate/notes",
} as never;

</storycode:copy>
<a id="f-065ce188ea68-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-065ce188ea68-expression_statement-3">
describe("the machine's own templates", () => {
  it("lists what dotnet has, grouped by its own tags", async () => {
    ready();
    // Twice on purpose: the row in the list, and the heading of the form it
    // opened on.
    expect((await screen.findAllByText("Console App")).length).toBe(2);
    expect(screen.getByText("ASP.NET Core Web API")).toBeTruthy();
    expect(screen.getByText("Common")).toBeTruthy();
    expect(screen.getByText("Web")).toBeTruthy();
    // Which SDK is answering is not something to go looking for.
    expect(screen.getByText("10.0.111")).toBeTruthy();
  });

  it("draws the template's own options as fields", async () => {
    ready();
    await screen.findByText("-f, --framework");
    // A choice the person may decline to make at all.
    expect(await screen.findByText("(leave to dotnet)")).toBeTruthy();
    expect(await screen.findByText(/Target net9\.0/)).toBeTruthy();
  });

  it("re-reads the options when the language changes", async () => {
    ready();
    await screen.findByText("-f, --framework");
    expect(api.scaffoldOptions).toHaveBeenCalledWith("console", "C#", "dotnet");

    fireEvent.change(screen.getByDisplayValue("C#"), { target: { value: "F#" } });
    await waitFor(() =>
      expect(api.scaffoldOptions).toHaveBeenCalledWith("console", "F#", "dotnet"),
    );
  });
});

</storycode:copy>
<a id="f-065ce188ea68-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-065ce188ea68-expression_statement-4">
describe("the fields the template does not own", () => {
  it("follows the project name until the person takes the wheel", async () => {
    ready();
    await screen.findByDisplayValue("Greeter");

    fireEvent.change(screen.getByDisplayValue("Greeter"), {
      target: { value: "Widgets" },
    });
    expect(screen.getByDisplayValue("widgets")).toBeTruthy();

    // Once touched, the folder is theirs: renaming must not overwrite it.
    fireEvent.change(screen.getByDisplayValue("widgets"), {
      target: { value: "apps/mine" },
    });
    fireEvent.change(screen.getByDisplayValue("Widgets"), {
      target: { value: "Gadgets" },
    });
    expect(screen.getByDisplayValue("apps/mine")).toBeTruthy();
  });

  it("hands over the terminal it started, and closes", async () => {
    // The button does not wait for a commit: it starts the scaffolder in a
    // terminal and gets out of the way.
    // docs/guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal.md
    const { onStarted, onClose } = ready();
    vi.mocked(api.scaffoldCreate).mockResolvedValue(STARTED);
    await screen.findByText("-f, --framework");
    // The preview is the commit, trailers and all.
    await screen.findByText(/Hick-Recipe: dotnet new console -o greeter/);

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() => expect(onStarted).toHaveBeenCalledWith(STARTED));
    expect(onClose).toHaveBeenCalled();

    const [spec] = vi.mocked(api.scaffoldCreate).mock.calls[0];
    expect(spec.template).toBe("console");
    expect(spec.name).toBe("Greeter");
    expect(spec.output).toBe("greeter");
    // The location is a field, and it is sent: a project is made where the
    // person said, not inside the folder the app happens to have open.
    expect(spec.location).toBe("/home/nate/notes");
    expect(spec.image).toBe("mcr.microsoft.com/dotnet/sdk:10.0");
    // Only what was decided: --framework was left at its default and is not
    // written; --no-restore starts on here and is.
    expect(spec.options).toEqual([{ flag: "--no-restore" }]);
  });

  it("puts the platform's own chooser behind the ellipsis", async () => {
    // The app has no `@tauri-apps/api` and no `invoke`, so the picker is a
    // route: the page asks the engine, and the engine asks the program
    // hosting it. See crates/hickory-cli/src/serve/shell.rs.
    ready();
    vi.mocked(api.pickFolder).mockResolvedValue({ path: "/home/nate/src/apps" });
    await screen.findByText("-f, --framework");

    fireEvent.click(screen.getByRole("button", { name: "Choose a folder" }));
    // Opened where the field already points, not at some default.
    await waitFor(() =>
      expect(api.pickFolder).toHaveBeenCalledWith("/home/nate/notes"),
    );
    await waitFor(() =>
      expect(
        (screen.getByLabelText("Location") as HTMLInputElement).value,
      ).toBe("/home/nate/src/apps"),
    );
  });

  it("keeps the location when the chooser is cancelled", async () => {
    // A cancel is `{path: null}`, an ordinary answer. Nothing changes and
    // nothing is said — a person who closed a dialog does not need telling.
    ready();
    vi.mocked(api.pickFolder).mockResolvedValue({ path: null });
    await screen.findByText("-f, --framework");
    fireEvent.click(screen.getByRole("button", { name: "Choose a folder" }));
    await waitFor(() => expect(api.pickFolder).toHaveBeenCalled());
    expect((screen.getByLabelText("Location") as HTMLInputElement).value).toBe(
      "/home/nate/notes",
    );
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("draws no ellipsis where there is nothing behind it", async () => {
    // `hick up` in a browser: the location can only be typed.
    vi.mocked(api.scaffoldTemplates).mockResolvedValue({
      ...CATALOG,
      can_pick_folder: false,
    });
    vi.mocked(api.scaffoldOptions).mockResolvedValue(DETAIL);
    vi.mocked(api.scaffoldPreview).mockResolvedValue({
      output: "greeter",
      folder: "/home/nate/notes/greeter",
      repository: "/home/nate/notes",
      command: "dotnet new console -o greeter -n Greeter",
      message: "Scaffold Greeter with `dotnet new console`\n",
    });
    render(<NewProjectDialog onStarted={vi.fn()} onClose={vi.fn()} />);
    await screen.findByText("-f, --framework");
    expect(screen.queryByRole("button", { name: "Choose a folder" })).toBeNull();
    expect(screen.getByLabelText("Location")).toBeTruthy();
  });

  it("makes a project anywhere on the machine", async () => {
    const { onStarted } = ready();
    vi.mocked(api.scaffoldCreate).mockResolvedValue(STARTED);
    await screen.findByText("-f, --framework");

    fireEvent.change(screen.getByLabelText("Location"), {
      target: { value: "/home/nate/src" },
    });
    // The dialog shows where the two fields land together, so the person can
    // read the answer rather than assemble it.
    await screen.findByText("/home/nate/src/greeter");

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() => expect(onStarted).toHaveBeenCalled());
    const [spec] = vi.mocked(api.scaffoldCreate).mock.calls[0];
    expect(spec.location).toBe("/home/nate/src");
    expect(spec.output).toBe("greeter");
  });

  it("names the repository that will record it when it is not the open one", async () => {
    ready();
    vi.mocked(api.scaffoldPreview).mockResolvedValue({
      output: "apps/greeter",
      folder: "/home/nate/src/apps/greeter",
      repository: "/home/nate/src",
      command: "dotnet new console -o apps/greeter -n Greeter",
      message: "Scaffold Greeter with `dotnet new console`\n",
    });
    fireEvent.change(await screen.findByLabelText("Location"), {
      target: { value: "/home/nate/src/apps" },
    });
    await screen.findByText(/not the folder you have open/);
  });

  it("shows the failure and stays open rather than losing the form", async () => {
    const { onClose } = ready();
    vi.mocked(api.scaffoldCreate).mockRejectedValue(
      new Error("`greeter/` already exists and is not empty."),
    );
    await screen.findByText("-f, --framework");

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await screen.findByText(/already exists/);
    expect(onClose).not.toHaveBeenCalled();
  });

  it("ticks 'create a git repository' by itself when the location has none", async () => {
    // The repository is a checkbox on the form, not a screen after a refusal:
    // by the time you have typed a location and a name you have said what you
    // want. It is ticked FOR you and stays yours to untick.
    ready();
    vi.mocked(api.scaffoldCreate).mockResolvedValue(STARTED);
    await screen.findByText("-f, --framework");
    // Inside a repository already: nothing to make, and it says which one.
    // Waited for, because it is the preview that knows — the checkbox has no
    // opinion of its own until the server has resolved the location.
    await screen.findByText(/Already in one:/);
    const box = screen.getByRole("checkbox", { name: /Create a git repository/ });
    expect((box as HTMLInputElement).checked).toBe(false);
    expect((box as HTMLInputElement).disabled).toBe(true);

    vi.mocked(api.scaffoldPreview).mockResolvedValue({
      output: "greeter",
      folder: "/home/nate/src/fresh/greeter",
      repository: null,
      needs_repository: "/home/nate/src/fresh",
      problem: "not inside a git repository",
      command: "dotnet new console -o greeter -n Greeter",
      message: "Scaffold Greeter with `dotnet new console`\n",
    });
    fireEvent.change(screen.getByLabelText("Location"), {
      target: { value: "/home/nate/src/fresh" },
    });
    await waitFor(() =>
      expect(
        (screen.getByRole("checkbox", {
          name: /Create a git repository/,
        }) as HTMLInputElement).checked,
      ).toBe(true),
    );

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() => expect(api.scaffoldCreate).toHaveBeenCalled());
    const [spec] = vi.mocked(api.scaffoldCreate).mock.calls[0];
    expect(spec.init_repository).toBe(true);
  });

  it("leaves the repository alone once the person unticks it", async () => {
    ready();
    vi.mocked(api.scaffoldCreate).mockRejectedValue(
      new ApiError(422, "that location is not inside a git repository", {
        missing: "repository",
        path: "/home/nate/src/fresh",
      }),
    );
    vi.mocked(api.scaffoldPreview).mockResolvedValue({
      output: "greeter",
      folder: "/home/nate/src/fresh/greeter",
      repository: null,
      needs_repository: "/home/nate/src/fresh",
      command: "dotnet new console -o greeter -n Greeter",
      message: "Scaffold Greeter with `dotnet new console`\n",
    });
    fireEvent.change(await screen.findByLabelText("Location"), {
      target: { value: "/home/nate/src/fresh" },
    });
    const box = await screen.findByRole("checkbox", {
      name: /Create a git repository/,
    });
    await waitFor(() => expect((box as HTMLInputElement).checked).toBe(true));
    fireEvent.click(box);
    // Untouched by the next keystroke: a derived answer stops deriving the
    // moment somebody decides for themselves.
    fireEvent.change(screen.getByDisplayValue("Greeter"), {
      target: { value: "Widgets" },
    });
    await waitFor(() => expect((box as HTMLInputElement).checked).toBe(false));

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    // The server still refuses by type, and its sentence is what is shown.
    await screen.findByText(/not inside a git repository/);
    const [spec] = vi.mocked(api.scaffoldCreate).mock.calls[0];
    expect(spec.init_repository).toBe(false);
  });

  it("opens a project made elsewhere, and leaves one made here alone", async () => {
    ready();
    vi.mocked(api.scaffoldCreate).mockResolvedValue(STARTED);
    await screen.findByText("-f, --framework");
    // Inside the folder this window already shows: the tree will show it, so
    // there is nothing to open.
    const open = screen.getByRole("checkbox", {
      name: /Open the project when it is made/,
    });
    expect((open as HTMLInputElement).checked).toBe(false);

    vi.mocked(api.scaffoldPreview).mockResolvedValue({
      output: "greeter",
      folder: "/home/nate/src/greeter",
      repository: "/home/nate/src",
      command: "dotnet new console -o greeter -n Greeter",
      message: "Scaffold Greeter with `dotnet new console`\n",
    });
    fireEvent.change(screen.getByLabelText("Location"), {
      target: { value: "/home/nate/src" },
    });
    await waitFor(() => expect((open as HTMLInputElement).checked).toBe(true));

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() => expect(api.scaffoldCreate).toHaveBeenCalled());
    expect(vi.mocked(api.scaffoldCreate).mock.calls[0][0].open).toBe("new-window");
  });

  it("takes over this window when the new-window box is cleared", async () => {
    ready();
    vi.mocked(api.scaffoldCreate).mockResolvedValue(STARTED);
    await screen.findByText("-f, --framework");
    fireEvent.click(
      screen.getByRole("checkbox", { name: /Open the project when it is made/ }),
    );
    const inNew = screen.getByRole("checkbox", { name: /in a new window/ });
    expect((inNew as HTMLInputElement).checked).toBe(true);
    fireEvent.click(inNew);
    // Said plainly, because it costs you the terminal you were just reading.
    await screen.findByText(/every tab in this window goes/);

    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() => expect(api.scaffoldCreate).toHaveBeenCalled());
    expect(vi.mocked(api.scaffoldCreate).mock.calls[0][0].open).toBe("this-window");
  });

  it("asks for no window at all when the project is not being opened", async () => {
    const { onStarted } = ready();
    vi.mocked(api.scaffoldCreate).mockResolvedValue(STARTED);
    await screen.findByText("-f, --framework");
    expect(
      (screen.getByRole("checkbox", { name: /in a new window/ }) as HTMLInputElement)
        .disabled,
    ).toBe(true);
    fireEvent.click(screen.getByRole("button", { name: "Create project" }));
    await waitFor(() => expect(onStarted).toHaveBeenCalled());
    expect(vi.mocked(api.scaffoldCreate).mock.calls[0][0].open).toBe("none");
  });

});

</storycode:copy>
<a id="f-065ce188ea68-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-065ce188ea68-expression_statement-5">
describe("a machine with no SDK", () => {
  it("gets its own screen, keyed off the field and not the wording", async () => {
    // The reason the route carries `missing: "dotnet"` at all: a reworded
    // message must not be able to take this screen away.
    vi.mocked(api.scaffoldTemplates).mockRejectedValue(
      new ApiError(422, "some sentence nobody should be matching on", {
        missing: "dotnet",
      }),
    );
    render(<NewProjectDialog onStarted={vi.fn()} onClose={vi.fn()} />);

    await screen.findByText("No .NET SDK on this machine");
    // A sentence and a link, not a button: this product has no catalogue for
    // a platform install and no business fetching one.
    expect(screen.queryByRole("button", { name: /install/i })).toBeNull();
    expect(screen.getByRole("link", { name: "the .NET SDK download page" }).getAttribute("href")).toBe("https://dotnet.microsoft.com/download");
  });

  it("reports any other failure as itself", async () => {
    vi.mocked(api.scaffoldTemplates).mockRejectedValue(
      new ApiError(500, "the template listing did not finish", {}),
    );
    render(<NewProjectDialog onStarted={vi.fn()} onClose={vi.fn()} />);
    await screen.findByText("Could not read the templates");
  });
});

</storycode:copy>
<a id="f-065ce188ea68-expression_statement-6"></a>
### `expression_statement`

Protects docs/guarantees/authoring/a-new-project-uses-the-toolchain-the-language-uses.md

<storycode:copy id="f-065ce188ea68-expression_statement-6">
// Protects docs/guarantees/authoring/a-new-project-uses-the-toolchain-the-language-uses.md
describe("choosing which scaffolder", () => {
  const UV = {
    id: "uv",
    label: "Python (uv)",
    language: "Python",
    version: "uv 0.11.7",
    templated: false,
  };
  const BOTH: ScaffoldCatalog = {
    ...CATALOG,
    toolchains: [CATALOG.toolchains[0], UV],
  };
  const UV_CATALOG: ScaffoldCatalog = {
    ...BOTH,
    kind: "uv",
    sdk_version: "uv 0.11.7",
    image: "ghcr.io/astral-sh/uv:0.11.7",
    templates: [
      {
        short_names: ["init"],
        name: "Python project",
        languages: ["Python"],
        default_language: "Python",
        tags: ["Python"],
      },
    ],
  };
  const UV_DETAIL: ScaffoldTemplateDetail = {
    title: "Python (uv) (uv init)",
    author: "",
    description: "Create a new project",
    other_languages: [],
    options: [
      {
        names: ["--lib"],
        flag: "--lib",
        kind: "bool",
        choices: [],
        default: null,
        description: "Create a project for a library",
        enabled_if: null,
      },
    ],
  };

  it("draws no picker on a machine with one scaffolder", async () => {
    ready();
    await screen.findByText("-f, --framework");
    expect(screen.queryByRole("radiogroup", { name: "Toolchain" })).toBeNull();
  });

  it("offers every scaffolder this machine has, and no others", async () => {
    vi.mocked(api.scaffoldTemplates).mockResolvedValue(BOTH);
    vi.mocked(api.scaffoldOptions).mockResolvedValue(DETAIL);
    render(<NewProjectDialog onStarted={vi.fn()} onClose={vi.fn()} />);
    await screen.findByText("-f, --framework");
    const group = screen.getByRole("radiogroup", { name: "Toolchain" });
    const labels = within(group)
      .getAllByRole("radio")
      .map((button) => button.textContent);
    expect(labels).toEqual([".NET", "Python (uv)"]);
    expect(
      within(group).getByRole("radio", { name: ".NET" }).getAttribute("aria-checked"),
    ).toBe("true");
  });

  it("asks the server again, for the scaffolder that was picked", async () => {
    vi.mocked(api.scaffoldTemplates).mockResolvedValue(BOTH);
    vi.mocked(api.scaffoldOptions).mockResolvedValue(DETAIL);
    render(<NewProjectDialog onStarted={vi.fn()} onClose={vi.fn()} />);
    await screen.findByText("-f, --framework");

    vi.mocked(api.scaffoldTemplates).mockResolvedValue(UV_CATALOG);
    vi.mocked(api.scaffoldOptions).mockResolvedValue(UV_DETAIL);
    fireEvent.click(screen.getByRole("radio", { name: "Python (uv)" }));

    await waitFor(() =>
      expect(api.scaffoldTemplates).toHaveBeenCalledWith("uv"),
    );
    // And the form is uv's own, not .NET's carried over: `--no-restore` is a
    // flag uv has never heard of.
    await screen.findByText("--lib");
    expect(screen.queryByText("-f, --framework")).toBeNull();
    await waitFor(() =>
      expect(api.scaffoldOptions).toHaveBeenCalledWith("init", "Python", "uv"),
    );
  });

  it("names the tool that is not installed, rather than always .NET", async () => {
    vi.mocked(api.scaffoldTemplates).mockRejectedValue(
      new ApiError(422, "no `uv` on this machine's PATH", { missing: "uv" }),
    );
    render(<NewProjectDialog onStarted={vi.fn()} onClose={vi.fn()} />);
    expect(await screen.findByText("No uv on this machine")).toBeTruthy();
  });
});

</storycode:copy>
<a id="f-065ce188ea68-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-065ce188ea68-expression_statement-7">
it("shares project fields and starts the exact selected spec as a real terminal job", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  vi.mocked(api.scaffoldTemplates).mockResolvedValue(CATALOG);
  vi.mocked(api.scaffoldOptions).mockResolvedValue(DETAIL);
  vi.mocked(api.scaffoldPreview).mockResolvedValue({ output: "agent-project", folder: "/home/nate/notes/agent-project", repository: CATALOG.location,
    command: "measured preview", message: "measured commit preview" });
  let complete!: (value: typeof STARTED) => void;
  vi.mocked(api.scaffoldCreate).mockImplementation(() => new Promise(done => { complete = done; }));
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const onStarted = vi.fn(), onClose = vi.fn();
  render(<UiActionContext.Provider value={registry}><NewProjectDialog onStarted={onStarted} onClose={onClose} /></UiActionContext.Provider>);
  await screen.findByText("-f, --framework");
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope,
    { id: "python:turn", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  await run("Set new project name", { value: "Agent Project" });
  expect(screen.getByDisplayValue("agent-project")).toBeTruthy();
  await run("Set new project option --framework", { value: "net9.0" });
  await run("Set new project folder name", { value: "custom-folder" });
  await run("Set new project name", { value: "Renamed Project" });
  expect(screen.getByDisplayValue("custom-folder")).toBeTruthy();
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:turn", kind: "agent" }, {
    handle: actions().find(action => action.label === "Create new project")!.handle, arguments: {}, request_id: "create",
  }); });
  await waitFor(() => expect(api.scaffoldCreate).toHaveBeenCalledTimes(1));
  expect(api.scaffoldCreate).toHaveBeenLastCalledWith(expect.objectContaining({ name: "Renamed Project", output: "custom-folder", options: expect.arrayContaining([{ flag: "--framework", value: "net9.0" }]) }));
  expect(await run("Create new project")).toMatchObject({ status: "disabled" });
  expect(await run("Close new project dialog")).toMatchObject({ status: "disabled" });
  fireEvent.change(screen.getByDisplayValue("custom-folder"), { target: { value: "late-edit" } });
  expect(screen.queryByDisplayValue("late-edit")).toBeNull();
  let result: unknown;
  await act(async () => { complete(STARTED); result = await pending; });
  expect(result).toMatchObject({ status: "result", outcome: { status: "running", job: "term-3", value: { committed: false } } });
  expect(onStarted).toHaveBeenCalledExactlyOnceWith(STARTED); expect(onClose).toHaveBeenCalledOnce();
});

</storycode:copy>
<a id="f-065ce188ea68-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-065ce188ea68-expression_statement-8">
it("refuses late template options after selecting a different template", async () => {
  const { act } = await import("@testing-library/react");
  vi.mocked(api.scaffoldTemplates).mockResolvedValue(CATALOG);
  const requests: { template: string; done: (value: ScaffoldTemplateDetail) => void }[] = [];
  vi.mocked(api.scaffoldOptions).mockImplementation(template => new Promise(done => requests.push({ template, done })));
  render(<NewProjectDialog onStarted={() => {}} onClose={() => {}} />);
  await screen.findByRole("option", { name: /Console App/ });
  await waitFor(() => expect(requests.some(request => request.template === "console")).toBe(true));
  fireEvent.click(screen.getByRole("option", { name: /ASP.NET Core Web API/ }));
  expect((screen.getByRole("button", { name: "Create project" }) as HTMLButtonElement).disabled).toBe(true);
  await waitFor(() => expect(requests.some(request => request.template === "webapi")).toBe(true));
  await act(async () => { for (const request of requests.filter(request => request.template === "console")) request.done({ ...DETAIL, description: "Late console options" }); });
  expect(screen.queryByText("Late console options")).toBeNull();
  const current = requests.filter(request => request.template === "webapi").at(-1)!;
  await act(async () => current.done({ ...DETAIL, description: "Current Web API options" }));
  expect(screen.getByText("Current Web API options")).toBeTruthy();
});
</storycode:copy>

### Assemble `apps/web/src/components/NewProjectDialog.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/NewProjectDialog.test.tsx"><storycode:paste select="#f-065ce188ea68-expression_statement"/><storycode:paste select="#f-065ce188ea68-import_statement"/><storycode:paste select="#f-065ce188ea68-import_statement-2"/><storycode:paste select="#f-065ce188ea68-import_statement-3"/><storycode:paste select="#f-065ce188ea68-expression_statement-2"/><storycode:paste select="#f-065ce188ea68-CATALOG"/><storycode:paste select="#f-065ce188ea68-DETAIL"/><storycode:paste select="#f-065ce188ea68-ready"/><storycode:paste select="#f-065ce188ea68-STARTED"/><storycode:paste select="#f-065ce188ea68-expression_statement-3"/><storycode:paste select="#f-065ce188ea68-expression_statement-4"/><storycode:paste select="#f-065ce188ea68-expression_statement-5"/><storycode:paste select="#f-065ce188ea68-expression_statement-6"/><storycode:paste select="#f-065ce188ea68-expression_statement-7"/><storycode:paste select="#f-065ce188ea68-expression_statement-8"/></storycode:file>


</storycode:doc>
