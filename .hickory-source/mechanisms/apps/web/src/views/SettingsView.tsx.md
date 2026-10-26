<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/SettingsView.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-38894484eb8f"></a>
## `apps/web/src/views/SettingsView.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Markdown Paragraphs Unwrap By Default](../../../../../guarantees/authoring/paragraphs-unwrap-by-default/contract.md), [Settings preserves the open workspace](../../../../../guarantees/authoring/settings-preserves-the-open-workspace/contract.md), [The editor keeps its space](../../../../../guarantees/authoring/the-editor-keeps-its-space/contract.md), [Word Navigation Is The Person's Choice](../../../../../guarantees/authoring/word-navigation-is-the-persons-choice/contract.md), [Save Can Format First, With The File's Own Formatter](../../../../../guarantees/editor-intelligence/save-can-format-first/contract.md), [Show the requested connections without changing their meaning](../../../../../guarantees/lineage/a-ribbon-is-drawn-where-it-is-asked-for/contract.md).

<a id="f-38894484eb8f-KeyRow"></a>
### `KeyRow`

<storycode:copy id="f-38894484eb8f-KeyRow">
import { UiCheckbox } from "../components/UiCheckbox";
import { loadEditorAppearance, saveEditorAppearance } from "../lib/editorAppearance";
import { CommandPathSettings } from "./CommandPathSettings";
// Settings ("#/settings"): LLM API keys, plus Appearance.
//
// The server is the truth about what keys are configured; this view only ever
// sees whether a key exists and a masked hint (GET cannot return a full key),
// and PUTs exactly the providers the user touched — a string to set, null to
// clear. Everything stays on this machine: the keys live in a local file the
// engine reads, not in any account.
//
// Appearance holds the display preferences that used to sit in the document
// toolbar: lineage style and visibility, tab placement, channel width (localStorage, per
// browser — lib/ribbonStyle.ts and friends) and the custom window title
// (persisted server-side as ui.json so the desktop shell can read it at
// launch). Settings leaves open editor buffers mounted while it is shown.

import { useEffect, useId, useState } from "react";
import { useUiCommands } from "../lib/uiActionBridge";

import { api } from "../api/client";
import { setFormatOnSave } from "../lib/formatOnSave";
import { loadUnwrapParagraphs, saveUnwrapParagraphs } from "../lib/unwrapParagraphs";
import { KeyboardSection } from "./KeyboardSection";
import { AgentSettings } from "./AgentSettings";
import { navigate } from "../router";
import type { ProviderId, ProviderKey } from "../api/types";
import {
  loadRibbonStyle,
  saveRibbonStyle,
  type RibbonStyle,
} from "../lib/ribbonStyle";
import {
  loadRibbonVisibility,
  saveRibbonVisibility,
  type RibbonVisibility,
} from "../lib/ribbonVisibility";
import { loadTabStyle, saveTabStyle, type TabStyle } from "../lib/tabStyle";
import {
  CHANNEL_WIDTH_PRESETS,
  loadChannelWidth,
  saveChannelWidth,
} from "../lib/channelWidth";
import { applyTheme, loadTheme, saveTheme, type Theme } from "../lib/theme";
import { loadWordMotion, saveWordMotion, type WordMotion } from "../lib/wordMotion";
import { saveFilesWindows, useFilesWindows } from "../lib/filesWindows";
import { saveFilesParallax, useFilesParallax } from "../lib/filesParallax";
import { saveSpatialSides, useSpatialSides } from "../lib/spatialSides";

function KeyRow({
  provider,
  busy,
  onSave,
  onClear,
}: {
  provider: ProviderKey;
  busy: boolean;
  onSave: (id: ProviderId, key: string) => void;
  onClear: (id: ProviderId) => void;
}) {
  // The draft is row-local: PUTting only what the user changed falls out of
  // each row owning exactly its own unsaved text.
  const [draft, setDraft] = useState("");

  return (
    <form
      className="settings-row"
      onSubmit={(event) => {
        event.preventDefault();
        const key = draft.trim();
        if (!key) return;
        onSave(provider.id, key);
        setDraft("");
      }}
    >
      <div className="settings-row__who">
        <span className="settings-row__label">{provider.label}</span>
        <span
          className={`settings-row__state mono ${provider.configured ? "is-configured" : ""}`}
        >
          {provider.configured
            ? `configured${provider.masked ? ` · ${provider.masked}` : ""}`
            : "not set"}
        </span>
      </div>
      <input
        type="password"
        className="settings-row__input mono"
        aria-label={`${provider.label} API key`}
        placeholder={provider.configured ? "Replace key" : "Paste key"}
        autoComplete="off"
        value={draft}
        onChange={(event) => setDraft(event.target.value)}
      />
      <div className="settings-row__actions">
        <button className="btn" type="submit" disabled={busy || !draft.trim()}>
          Save
        </button>
        {provider.configured && (
          <button
            className="btn btn-quiet"
            type="button"
            disabled={busy}
            onClick={() => onClear(provider.id)}
          >
            Clear
          </button>
        )}
      </div>
    </form>
  );
}

</storycode:copy>
<a id="f-38894484eb8f-ChoiceRow"></a>
### `ChoiceRow`

One Appearance row: a label and a group of exclusive choices.

<storycode:copy id="f-38894484eb8f-ChoiceRow">
/** One Appearance row: a label and a group of exclusive choices. */
function ChoiceRow<T extends string | number>({
  label,
  choices,
  value,
  onPick,
}: {
  label: string;
  choices: readonly { value: T; label: string; title: string }[];
  value: T;
  onPick: (value: T) => void;
}) {
  const instance = useId();
  useUiCommands([{ key: `settings-choice:${instance}`, command: {
    label: `Set ${label}`, parent: "settings-choices", subject: `settings-choice:${instance}`, parameters: { choice: { type: "string", maxLength: 128, choices: choices.map(item => String(item.value)) } }, prerequisites: [],
    read: () => ({ revision: JSON.stringify([value, choices]), enabled: true, value: { selected: value, choices: choices.map(item => ({ value: String(item.value), label: item.label, meaning: item.title })) } }),
    execute: async args => {
      const selected = choices.find(item => String(item.value) === args.choice);
      if (!selected) return { status: "unavailable", reason: "This setting choice is no longer available" };
      onPick(selected.value); return { status: "completed", value: { selected: selected.value } };
    },
  } }]);
  return (
    <div className="settings-row settings-row--appearance">
      <div className="settings-row__who">
        <span className="settings-row__label">{label}</span>
      </div>
      <div className="ribbon-style-toggle" role="group" aria-label={label}>
        {choices.map((choice) => (
          <button
            key={String(choice.value)}
            type="button"
            aria-pressed={value === choice.value}
            data-tip={choice.title}
            onClick={() => onPick(choice.value)}
          >
            {choice.label}
          </button>
        ))}
      </div>
    </div>
  );
}

</storycode:copy>
<a id="f-38894484eb8f-AppearanceSection"></a>
### `AppearanceSection`

The display preferences. The toggles write localStorage on the spot
(the same libs the workspace reads at mount); the window title round-trips
through GET/PUT /api/settings/ui so the desktop shell can also read it at
launch for the native title bar.

<storycode:copy id="f-38894484eb8f-AppearanceSection">
/**
 * The display preferences. The toggles write localStorage on the spot
 * (the same libs the workspace reads at mount); the window title round-trips
 * through GET/PUT /api/settings/ui so the desktop shell can also read it at
 * launch for the native title bar.
 */
function AppearanceSection() {
  const [editorAppearance, setEditorAppearance] = useState(loadEditorAppearance);
  const spatialSides = useSpatialSides();
  const [theme, setTheme] = useState<Theme>(() => loadTheme());
  const [ribbonStyle, setRibbonStyle] = useState<RibbonStyle>(() => loadRibbonStyle());
  const [ribbonVisibility, setRibbonVisibility] = useState<RibbonVisibility>(
    () => loadRibbonVisibility(),
  );
  const [tabStyle, setTabStyle] = useState<TabStyle>(() => loadTabStyle());
  const [channelWidth, setChannelWidth] = useState<number>(() => loadChannelWidth());

  // The window title as the server knows it, and the unsaved draft.
  const [savedTitle, setSavedTitle] = useState<string | null>(null);
  const [titleDraft, setTitleDraft] = useState("");
  const [titleError, setTitleError] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    api.settingsUi().then(
      (ui) => {
        if (!live) return;
        setSavedTitle(ui.window_title);
        setTitleDraft(ui.window_title ?? "");
      },
      (e) => {
        if (live) setTitleError(e instanceof Error ? e.message : String(e));
      },
    );
    return () => {
      live = false;
    };
  }, []);

  const applyTitle = async (value: string | null) => {
    setTitleError(null);
    try {
      const ui = await api.saveSettingsUi({ window_title: value });
      setSavedTitle(ui.window_title); setTitleDraft(ui.window_title ?? "");
      return { saved: ui.window_title };
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e); setTitleError(error); return { error };
    }
  };
  const titleRead = () => ({ revision: JSON.stringify([savedTitle, titleDraft, titleError]), enabled: true, value: { saved: savedTitle, draft: titleDraft, error: titleError } });
  useUiCommands([
    { key: "settings/title-input", command: { label: "Set window title draft", parent: "settings-appearance", subject: "settings-window-title", parameters: { title: { type: "string", maxLength: 4096 } }, prerequisites: [], read: titleRead, execute: async args => { setTitleDraft(String(args.title)); return { status: "completed", value: { saved: false } }; } } },
    ...([false, true].map(clear => ({ key: `settings/title-${clear ? "clear" : "save"}`, command: {
      label: clear ? "Clear custom window title" : "Save custom window title", parent: "settings-appearance", subject: "settings-window-title", parameters: {}, prerequisites: [], read: titleRead,
      execute: async () => {
        const result = await applyTitle(clear ? null : titleDraft.trim() || null);
        return result.error ? { status: "unavailable" as const, reason: result.error } : { status: "completed" as const, value: { saved: result.saved ?? null } };
      },
    } }))),
  ]);

  return (
    <section className="settings__appearance" aria-label="Appearance">
      <h2 className="settings__section-title">Appearance</h2>
      <div className="settings__rows">
        <ChoiceRow
          label="Theme"
          value={theme}
          onPick={(next) => {
            setTheme(next);
            saveTheme(next);
            applyTheme(next);
          }}
          choices={[
            { value: "dark", label: "Dark", title: "Cool, editor-grade dark (default)" },
            { value: "warm-dark", label: "Warm dark", title: "The warm sepia dark palette" },
            { value: "light", label: "Light", title: "Warm paper light palette" },
          ]}
        />
        <ChoiceRow
          label="Lineage style"
          value={ribbonStyle}
          onPick={(style) => {
            setRibbonStyle(style);
            saveRibbonStyle(style);
          }}
          choices={[
            {
              value: "bands",
              label: "Ribbons",
              title: "Draw lineage as filled ribbons between panes",
            },
            {
              value: "braces",
              label: "Braces",
              title: "Draw lineage as curly braces joined by a thin line",
            },
          ]}
        />
        <ChoiceRow
          label="Lineage visibility"
          value={ribbonVisibility}
          onPick={(next) => {
            setRibbonVisibility(next);
            saveRibbonVisibility(next);
          }}
          choices={[
            {
              value: "caret",
              label: "With the caret",
              title:
                "Draw a connection only while the caret is in one of the blocks it joins",
            },
            {
              value: "always",
              label: "Always",
              title: "Draw every connection all the time",
            },
          ]}
        />
        <ChoiceRow
          label="Editor density"
          value={editorAppearance.density}
          onPick={(density) => {
            const next = { ...editorAppearance, density };
            setEditorAppearance(next); saveEditorAppearance(next);
          }}
          choices={[
            { value: "compact", label: "Compact", title: "More text on screen, with smaller margins and tighter line spacing" },
            { value: "comfortable", label: "Comfortable", title: "Generous margins and line spacing for reading prose" },
          ]}
        />
        <div className="settings-row settings-row--appearance">
          <div className="settings-row__who"><label className="settings-row__label" htmlFor="touch-file-rows">Large Files touch targets</label></div>
          <div className="settings-row__actions"><UiCheckbox label="Large Files touch targets" id="touch-file-rows" checked={editorAppearance.touchTargets} onCheckedChange={enabled => {
            const next = { ...editorAppearance, touchTargets: enabled };
            setEditorAppearance(next); saveEditorAppearance(next);
          }} /></div>
        </div>
        <div className="settings-row settings-row--appearance">
          <div className="settings-row__who"><label className="settings-row__label" htmlFor="right-line-numbers">Right-hand line numbers</label></div>
          <div className="settings-row__actions"><UiCheckbox label="Right-hand line numbers" id="right-line-numbers" checked={editorAppearance.rightLineNumbers} onCheckedChange={enabled => {
            const next = { ...editorAppearance, rightLineNumbers: enabled };
            setEditorAppearance(next); saveEditorAppearance(next);
          }} /></div>
        </div>
        <ChoiceRow
          label="Tab placement"
          value={tabStyle}
          onPick={(style) => {
            setTabStyle(style);
            saveTabStyle(style);
          }}
          choices={[
            { value: "top", label: "Top", title: "Tabs across the top of each pane" },
            {
              value: "side",
              label: "Side",
              title: "Tabs down a left sidebar, grouped by folder",
            },
          ]}
        />
        <ChoiceRow
          label="Channel width"
          value={channelWidth}
          onPick={(px) => {
            setChannelWidth(px);
            saveChannelWidth(px);
          }}
          choices={CHANNEL_WIDTH_PRESETS.map((preset) => ({
            value: preset.px,
            label: preset.label,
            title: `${preset.label} channel between panes (${preset.px}px)`,
          }))}
        />
        {(["left", "right"] as const).map(side => <div key={side} className="settings-row settings-row--appearance">
          <div className="settings-row__who"><label className="settings-row__label" htmlFor={`spatial-${side}`}>{side === "left" ? "Left" : "Right"} drawer</label></div>
          <div className="settings-row__actions"><UiCheckbox label={`${side} drawer`} id={`spatial-${side}`} checked={spatialSides[side]} onCheckedChange={enabled => saveSpatialSides({ ...spatialSides, [side]: enabled })} /></div>
        </div>)}
        <p className="muted">Drawers appear only when they have content and start collapsed. Opening overlays the editor; pinning makes space beside it.</p>
        <form
          className="settings-row"
          onSubmit={(event) => {
            event.preventDefault();
            const title = titleDraft.trim();
            applyTitle(title === "" ? null : title);
          }}
        >
          <div className="settings-row__who">
            <span className="settings-row__label">Window title</span>
            <span className={`settings-row__state mono ${savedTitle ? "is-configured" : ""}`}>
              {savedTitle ? "custom" : "default (folder name)"}
            </span>
          </div>
          <input
            type="text"
            className="settings-row__input"
            aria-label="Window title"
            placeholder="Folder name (default)"
            value={titleDraft}
            onChange={(event) => setTitleDraft(event.target.value)}
          />
          <div className="settings-row__actions">
            <button className="btn" type="submit">
              Save
            </button>
            {savedTitle !== null && (
              <button className="btn btn-quiet" type="button" onClick={() => applyTitle(null)}>
                Clear
              </button>
            )}
          </div>
        </form>
        {titleError && <p className="error">{titleError}</p>}
      </div>
      <p className="muted settings__note">
        The window title falls back to the open folder's name, then the
        focused file. A custom title also names the desktop window at its
        next launch.
      </p>
    </section>
  );
}

</storycode:copy>
<a id="f-38894484eb8f-ExperimentsSection"></a>
### `ExperimentsSection`

<storycode:copy id="f-38894484eb8f-ExperimentsSection">
function ExperimentsSection() {
  const filesParallax = useFilesParallax();
  const filesWindows = useFilesWindows();
  return (
    <section aria-label="Experiments">
      <h2 className="settings__section-title">Experiments</h2>
      <div className="settings__rows">
        <div className="settings-row settings-row--appearance">
          <div className="settings-row__who"><label className="settings-row__label" htmlFor="files-windows">Show windows in Files pane</label></div>
          <div className="settings-row__actions">
            <UiCheckbox label="Show windows in Files pane" id="files-windows" checked={filesWindows} onCheckedChange={saveFilesWindows} />
            <span className="muted">Show related and pinned preview windows alongside the file list. Off by default.</span>
          </div>
        </div>
        <div className="settings-row settings-row--appearance">
          <div className="settings-row__who"><label className="settings-row__label" htmlFor="files-parallax">Files pane parallax</label></div>
          <div className="settings-row__actions">
            <UiCheckbox label="Files parallax" id="files-parallax" checked={filesParallax} onCheckedChange={saveFilesParallax} />
            <span className="muted">Move Files previews at a different rate while scrolling the editor. Off by default.</span>
          </div>
        </div>
      </div>
    </section>
  );
}

</storycode:copy>
<a id="f-38894484eb8f-EditingSection"></a>
### `EditingSection`

Editing behaviour, as opposed to how the app looks.
Word navigation is here rather than under Appearance because it
changes what a key DOES: a person hunting for it after Ctrl+→ overshot is
not looking under "Appearance". The keymap re-reads the stored value on
every Ctrl+arrow, so the change lands in editors that are already open
without navigating back — see editor/wordMotion.ts.

<storycode:copy id="f-38894484eb8f-EditingSection">
/**
 * Editing behaviour, as opposed to how the app looks.
 *
 * Word navigation is here rather than under Appearance because it
 * changes what a key DOES: a person hunting for it after Ctrl+→ overshot is
 * not looking under "Appearance". The keymap re-reads the stored value on
 * every Ctrl+arrow, so the change lands in editors that are already open
 * without navigating back — see editor/wordMotion.ts.
 */
function EditingSection() {
  const [unwrap, setUnwrap] = useState(loadUnwrapParagraphs);
  const [wordMotion, setWordMotion] = useState<WordMotion>(() => loadWordMotion());
  // Format on save, as the server knows it. Off until the read lands: the
  // default, and the honest answer while nothing is known.
  const [formatSave, setFormatSave] = useState(false);
  const [retainSavedDrafts, setRetainSavedDrafts] = useState(false);
  const [formatError, setFormatError] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);
  useEffect(() => {
    let live = true;
    api.settingsUi().then(
      (ui) => {
        if (!live) return;
        setFormatSave(ui.format_on_save === true);
        setRetainSavedDrafts(ui.retain_unsaved_saved_files === true);
        setFormatOnSave(ui.format_on_save === true);
        setLoaded(true);
      },
      (e) => {
        if (live) setFormatError(e instanceof Error ? e.message : String(e));
      },
    );
    return () => {
      live = false;
    };
  }, []);
  const applyFormatSave = (on: boolean) => {
    setFormatError(null);
    setFormatSave(on);
    api.saveSettingsUi({ format_on_save: on }).then(
      (ui) => {
        setFormatSave(ui.format_on_save === true);
        setFormatOnSave(ui.format_on_save === true);
      },
      (e) => {
        setFormatSave(!on);
        setFormatError(e instanceof Error ? e.message : String(e));
      },
    );
  };
  const applyRetainSavedDrafts = (on: boolean) => {
    setFormatError(null);
    setRetainSavedDrafts(on);
    api.saveSettingsUi({ retain_unsaved_saved_files: on }).then(
      (ui) => {
        setRetainSavedDrafts(ui.retain_unsaved_saved_files === true);
        if (!on) {
          void api.drafts().then(({ drafts }) =>
            Promise.all(
              drafts
                .filter((draft) => !draft.path.startsWith("untitled:"))
                .map((draft) => api.discardDraft(draft.path)),
            ),
          );
        }
      },
      (e) => {
        setRetainSavedDrafts(!on);
        setFormatError(e instanceof Error ? e.message : String(e));
      },
    );
  };
  useUiCommands([{ key: "settings/editing-state", command: {
    label: "Read editing preference state and errors", parent: "settings", subject: "settings-editing", parameters: {}, prerequisites: [],
    read: () => ({ revision: JSON.stringify([unwrap, wordMotion, formatSave, retainSavedDrafts, formatError, loaded]), enabled: true }),
    execute: async () => ({ status: "completed", value: { unwrap, word_motion: wordMotion, displayed_format_on_save: formatSave, displayed_retain_saved_drafts: retainSavedDrafts, initial_host_read_completed: loaded, error: formatError, scope: "Current displayed preferences; asynchronous saves can still be pending" } }),
  } }]);

  return (
    <section className="settings__appearance" aria-label="Editing">
      <h2 className="settings__section-title">Editing</h2>
      <div className="settings__rows">
        <div className="settings-row settings-row--appearance">
          <div className="settings-row__who">
            <label className="settings-row__label" htmlFor="unwrap-paragraphs">
              Unwrap Markdown paragraphs
            </label>
          </div>
          <div className="settings-row__actions">
            <UiCheckbox
              label="Unwrap Markdown paragraphs"
              id="unwrap-paragraphs"
              checked={unwrap}
              onCheckedChange={(enabled) => {
                setUnwrap(enabled);
                saveUnwrapParagraphs(enabled);
              }}
            />
            <span className="muted">
              Join wrapped paragraph lines when opening documents or pasting text,
              then wrap them visually at the ruler. Explicit line breaks, lists,
              code and document payloads keep their lines. On by default.
            </span>
          </div>
        </div>
        <ChoiceRow
          label="Word navigation"
          value={wordMotion}
          onPick={(motion) => {
            setWordMotion(motion);
            saveWordMotion(motion);
          }}
          choices={[
            {
              value: "word",
              label: "Whole words",
              title: "Ctrl+← and Ctrl+→ stop at spaces and punctuation",
            },
            {
              value: "subword",
              label: "Subwords",
              title:
                "Also stop inside an identifier: PascalCase, camelCase, snake_case",
            },
          ]}
        />
        <div className="settings-row settings-row--appearance">
          <div className="settings-row__who">
            <label className="settings-row__label" htmlFor="format-on-save">
              Format on save
            </label>
          </div>
          <div className="settings-row__actions">
            <UiCheckbox
              label="Format on save"
              id="format-on-save"
              checked={formatSave}
              onCheckedChange={applyFormatSave}
            />
            <span className="muted">
              Run the file’s formatter — rustfmt, black, prettier, whichever its
              language server offers — when you choose Save. Shift+Alt+F
              formats at any time.
            </span>
            {formatError && (
              <span className="error" role="alert">
                {formatError}
              </span>
            )}
          </div>
        </div>
        <div className="settings-row settings-row--appearance">
          <div className="settings-row__who">
            <label className="settings-row__label" htmlFor="retain-unsaved-saved-files">
              Retain unsaved changes
            </label>
          </div>
          <div className="settings-row__actions">
            <UiCheckbox
              label="Retain saved-file recovery drafts"
              id="retain-unsaved-saved-files"
              checked={retainSavedDrafts}
              onCheckedChange={applyRetainSavedDrafts}
            />
            <span className="muted">
              Recover unsaved changes after an unexpected interruption.
              New, unnamed documents always have crash recovery.
            </span>
          </div>
        </div>
      </div>
      <p className="muted settings__note">
        Subwords is what a code editor calls CamelHumps: <code>Ctrl+→</code> from
        the start of <code>XMLHttpRequest</code> stops at <code>Http</code>, then
        at <code>Request</code>. Shift extends the selection the same way. This
        is the motion you want in code and the wrong one in prose, and a{" "}
        <code>.md</code> document is both — so it is your choice rather than
        the file’s. On macOS the keys are Option+← and Option+→.
      </p>
    </section>
  );
}

</storycode:copy>
<a id="f-38894484eb8f-SettingsView"></a>
### `SettingsView`

<storycode:copy id="f-38894484eb8f-SettingsView">
export function SettingsView({ onBack = () => navigate("/") }: { onBack?: () => void } = {}) {
  const [providers, setProviders] = useState<ProviderKey[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [saved, setSaved] = useState<string | null>(null);

  useEffect(() => {
    const previousTitle = document.title;
    document.title = "Settings — Hickory Docs";
    return () => { document.title = previousTitle; };
  }, []);

  const reload = () =>
    api.settingsKeys().then(
      (r) => {
        setProviders(r.providers);
        setError(null);
      },
      (e) => setError(e instanceof Error ? e.message : String(e)),
    );

  useEffect(() => {
    void reload();
  }, []);

  // Set or clear one provider, then re-read: the server is the truth about
  // what is now configured (and about the masked hint it chose to show).
  const apply = async (id: ProviderId, value: string | null) => {
    setBusy(true);
    setSaved(null);
    try {
      await api.saveSettingsKeys({ [id]: value });
      await reload();
      setSaved(value === null ? `${id} key cleared.` : `${id} key saved — in effect now.`);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && !e.defaultPrevented) {
        e.preventDefault();
        onBack();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onBack]);

  const configured = (providers ?? []).filter((p) => p.configured);
  const settingsRevision = JSON.stringify([providers, error, busy, saved]);
  useUiCommands([
    { key: "settings/state", command: { label: "Read provider configuration status", parent: "settings", subject: "settings", parameters: {}, prerequisites: ["Only the host's masked status is available; secrets and unsaved key drafts are excluded"], read: () => ({ revision: settingsRevision, enabled: true }), execute: async () => ({ status: "completed", value: JSON.parse(JSON.stringify({ providers, error, busy, saved })) }) } },
    { key: "settings/refresh", command: { label: "Refresh provider configuration status", parent: "settings", subject: "settings", parameters: {}, prerequisites: [], read: () => ({ revision: settingsRevision, enabled: !busy }), execute: async () => { await reload(); return { status: "completed", value: { result: "Observe host configuration status or error" } }; } } },
    { key: "settings/back", command: { label: "Return from settings", parent: "settings", subject: "settings", parameters: {}, prerequisites: [], read: () => ({ revision: settingsRevision, enabled: true }), execute: async () => { onBack(); return { status: "completed", value: null }; } } },
    ...((providers ?? []).map(provider => ({ key: `settings/credential/${provider.id}`, command: {
      label: `Request credential change: ${provider.label}`, parent: "settings", subject: `credential:${provider.id}`, parameters: {}, prerequisites: ["Actual human credential input; never include a key in model context or action receipts"],
      read: () => ({ revision: settingsRevision, enabled: !busy, value: { configured: provider.configured, masked: provider.masked ?? null } }),
      execute: async () => ({ status: "requires_input" as const, request: `settings:credential:${provider.id}`, reason: `Use the ${provider.label} key row to enter or clear a credential. Then refresh provider configuration status to inspect the actual host result; no credential change has run.` }),
    } }))),
  ]);

  return (
    <div className="start settings">
      <div className="start__inner">
        <header className="settings__head">
          <h1 className="start__title">Settings</h1>
          <button
            className="btn"
            onClick={onBack}
            data-tip="Back to the document (Escape)"
          >
            ← Back
          </button>
        </header>
        <p className="start__lede">
          API keys for the AI agent. They are stored in a local file readable
          only by you, used only to call the provider you chose, and take
          effect immediately — no restart. Nothing else ever sees them.
        </p>

        {error && <p className="error">{error}</p>}
        {saved && (
          <p className="settings__saved" role="status">
            {saved}
          </p>
        )}

        {providers === null && !error && <p className="muted">Loading…</p>}

        {providers !== null && (
          <div className="settings__rows">
            {providers.map((provider) => (
              <KeyRow
                key={provider.id}
                provider={provider}
                busy={busy}
                onSave={(id, key) => void apply(id, key)}
                onClear={(id) => void apply(id, null)}
              />
            ))}
          </div>
        )}

        {providers !== null && (
          <p className="muted settings__note">
            {configured.length === 1
              ? `With exactly one key configured, the agent uses ${configured[0].label} automatically.`
              : "When exactly one provider has a key, the agent selects it automatically."}
          </p>
        )}

        <AppearanceSection />
        <EditingSection />
        <ExperimentsSection />
        <CommandPathSettings />
        <AgentSettings />
      <KeyboardSection />
      </div>
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/views/SettingsView.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/SettingsView.tsx"><storycode:paste select="#f-38894484eb8f-KeyRow"/><storycode:paste select="#f-38894484eb8f-ChoiceRow"/><storycode:paste select="#f-38894484eb8f-AppearanceSection"/><storycode:paste select="#f-38894484eb8f-ExperimentsSection"/><storycode:paste select="#f-38894484eb8f-EditingSection"/><storycode:paste select="#f-38894484eb8f-SettingsView"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-5bfac230ae5d"></a>
## `apps/web/src/views/SettingsView.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [Markdown Paragraphs Unwrap By Default](../../../../../guarantees/authoring/paragraphs-unwrap-by-default/contract.md), [The editor keeps its space](../../../../../guarantees/authoring/the-editor-keeps-its-space/contract.md), [Close or recover a document without losing the wrong draft](../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md), [Word Navigation Is The Person's Choice](../../../../../guarantees/authoring/word-navigation-is-the-persons-choice/contract.md), [Save Can Format First, With The File's Own Formatter](../../../../../guarantees/editor-intelligence/save-can-format-first/contract.md), [Show the requested connections without changing their meaning](../../../../../guarantees/lineage/a-ribbon-is-drawn-where-it-is-asked-for/contract.md).

<a id="f-5bfac230ae5d-providers"></a>
### `providers`

<storycode:copy id="f-5bfac230ae5d-providers">
import { FILES_WINDOWS_KEY, loadFilesWindows } from "../lib/filesWindows";
import { FILES_PARALLAX_KEY, loadFilesParallax } from "../lib/filesParallax";
import { EDITOR_APPEARANCE_KEY, loadEditorAppearance } from "../lib/editorAppearance";
import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { installMockHandler } from "../api/client";
import type { ProviderKey, SettingsKeysPatch, UiSettings } from "../api/types";
import { RIBBON_STYLE_KEY, loadRibbonStyle } from "../lib/ribbonStyle";
import {
  RIBBON_VISIBILITY_KEY,
  loadRibbonVisibility,
} from "../lib/ribbonVisibility";
import { TAB_STYLE_KEY, loadTabStyle } from "../lib/tabStyle";
import { CHANNEL_WIDTH_KEY, loadChannelWidth } from "../lib/channelWidth";
import { THEME_KEY, loadTheme } from "../lib/theme";
import { WORD_MOTION_KEY, loadWordMotion } from "../lib/wordMotion";
import { SPATIAL_SIDES_KEY, loadSpatialSides } from "../lib/spatialSides";
import { SettingsView } from "./SettingsView";
import { UNWRAP_PARAGRAPHS_KEY, loadUnwrapParagraphs } from "../lib/unwrapParagraphs";

// The backend contract (GET/PUT /api/settings/keys) is being added
// concurrently; these tests mock it at the client seam, which is also what
// keeps them honest about the one hard rule: GET never carries a full key.
let providers: ProviderKey[];
</storycode:copy>
<a id="f-5bfac230ae5d-puts"></a>
### `puts`

<storycode:copy id="f-5bfac230ae5d-puts">
let puts: SettingsKeysPatch[];
</storycode:copy>
<a id="f-5bfac230ae5d-uiSettings"></a>
### `uiSettings`

<storycode:copy id="f-5bfac230ae5d-uiSettings">
let uiSettings: UiSettings;
</storycode:copy>
<a id="f-5bfac230ae5d-uiPuts"></a>
### `uiPuts`

<storycode:copy id="f-5bfac230ae5d-uiPuts">
let uiPuts: UiSettings[];

</storycode:copy>
<a id="f-5bfac230ae5d-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-5bfac230ae5d-expression_statement">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares setting choices while keeping credential drafts out of agent observations", async () => {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><SettingsView /></UiActionContext.Provider>);
  await screen.findByLabelText("OpenAI API key");
  fireEvent.change(screen.getByLabelText("OpenAI API key"), { target: { value: "draft-secret-never-context" } });
  expect(JSON.stringify(registry.observe("settings"))).not.toContain("draft-secret-never-context");
  const choice = registry.observe("settings-choices").actions.find(action => action.label === "Set Theme")!;
  await act(async () => { expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: choice.handle, arguments: { choice: "light" }, request_id: "theme" })).toMatchObject({ status: "result" }); });
  expect(loadTheme()).toBe("light");
  const credential = registry.observe("settings").actions.find(action => action.label === "Request credential change: OpenAI")!;
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: credential.handle, arguments: {}, request_id: "credential" })).toMatchObject({ status: "result", outcome: { status: "requires_input", request: "settings:credential:openai" } });
  expect(puts).toEqual([]);
  const toggle = registry.observe("settings-toggles").actions.find(action => action.label === "Set Right-hand line numbers")!;
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: toggle.handle, arguments: { checked: true }, request_id: "right-numbers" }); });
  expect(loadEditorAppearance().rightLineNumbers).toBe(true);
  const draft = registry.observe("settings-appearance").actions.find(action => action.label === "Set window title draft")!;
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: draft.handle, arguments: { title: "Investigation" }, request_id: "title-draft" }); });
  const save = registry.observe("settings-appearance").actions.find(action => action.label === "Save custom window title")!;
  await act(async () => { expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: save.handle, arguments: {}, request_id: "title-save" })).toMatchObject({ status: "result", outcome: { value: { saved: "Investigation" } } }); });
  expect(uiPuts).toContainEqual({ window_title: "Investigation" });
});

</storycode:copy>
<a id="f-5bfac230ae5d-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-5bfac230ae5d-expression_statement-2">
beforeEach(() => {
  localStorage.removeItem(FILES_WINDOWS_KEY);
  localStorage.removeItem(FILES_PARALLAX_KEY);
  localStorage.removeItem(EDITOR_APPEARANCE_KEY);
  providers = [
    { id: "anthropic", label: "Anthropic", configured: true, masked: "sk-a…f3" },
    { id: "openai", label: "OpenAI", configured: false, masked: null },
    { id: "openrouter", label: "OpenRouter", configured: false, masked: null },
    { id: "deepseek", label: "DeepSeek", configured: false, masked: null },
    { id: "xai", label: "xAI", configured: false, masked: null },
  ];
  puts = [];
  localStorage.removeItem(SPATIAL_SIDES_KEY);
  localStorage.removeItem(UNWRAP_PARAGRAPHS_KEY);
  uiSettings = { window_title: null, format_on_save: false };
  uiPuts = [];
  localStorage.removeItem(RIBBON_STYLE_KEY);
  localStorage.removeItem(TAB_STYLE_KEY);
  localStorage.removeItem(CHANNEL_WIDTH_KEY);
  localStorage.removeItem(THEME_KEY);
  delete document.documentElement.dataset.theme;
  installMockHandler(async (method, path, body) => {
    if (path === "/api/settings/ui") {
      if (method === "PUT") {
        const next = body as UiSettings;
        uiPuts.push(next);
        uiSettings = { ...uiSettings, ...next };
      }
      return uiSettings;
    }
    if (path !== "/api/settings/keys") throw new Error(`unexpected ${method} ${path}`);
    if (method === "PUT") {
      const patch = body as SettingsKeysPatch;
      puts.push(patch);
      providers = providers.map((p) =>
        p.id in patch
          ? patch[p.id] === null
            ? { ...p, configured: false, masked: null }
            : { ...p, configured: true, masked: `${String(patch[p.id]).slice(0, 4)}…` }
          : p,
      );
    }
    return { providers };
  });
});

</storycode:copy>
<a id="f-5bfac230ae5d-expression_statement-3"></a>
### `expression_statement`

vitest runs without `globals`, so Testing Library's automatic cleanup never
registers itself.

<storycode:copy id="f-5bfac230ae5d-expression_statement-3">
// vitest runs without `globals`, so Testing Library's automatic cleanup never
// registers itself.
afterEach(() => {
  cleanup();
  installMockHandler(null as never);
});

</storycode:copy>
<a id="f-5bfac230ae5d-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-5bfac230ae5d-expression_statement-4">
describe("the settings view", () => {
  // Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
  it("remembers spacing, touch targets and right numbers independently", () => {
    const ui = render(<SettingsView />);
    expect(loadEditorAppearance()).toEqual({ density: "compact", touchTargets: false, rightLineNumbers: true });
    fireEvent.click(screen.getByRole("button", { name: "Comfortable" }));
    fireEvent.click(screen.getByRole("checkbox", { name: "Large Files touch targets" }));
    fireEvent.click(screen.getByRole("checkbox", { name: "Right-hand line numbers" }));
    expect(loadEditorAppearance()).toEqual({ density: "comfortable", touchTargets: true, rightLineNumbers: false });
    expect(document.documentElement.dataset.editorDensity).toBe("comfortable");
    expect(document.documentElement.dataset.rightLineNumbers).toBe("false");
    ui.unmount(); render(<SettingsView />);
    expect(screen.getByRole("button", { name: "Comfortable" }).getAttribute("aria-pressed")).toBe("true");
    expect((screen.getByRole("checkbox", { name: "Right-hand line numbers" }) as HTMLInputElement).checked).toBe(false);
  });
  it("defaults left context on and right context off, and remembers independent changes", () => {
    const ui = render(<SettingsView />);
    const left = () => screen.getByRole("checkbox", { name: "Left drawer" }) as HTMLInputElement;
    const right = () => screen.getByRole("checkbox", { name: "Right drawer" }) as HTMLInputElement;
    expect(left().checked).toBe(true); expect(right().checked).toBe(false);
    fireEvent.click(right()); expect(loadSpatialSides()).toEqual({ left: true, right: true });
    fireEvent.click(left()); expect(loadSpatialSides()).toEqual({ left: false, right: true });
    ui.unmount(); render(<SettingsView />);
    expect(left().checked).toBe(false); expect(right().checked).toBe(true);
  });
  // Guarantee: docs/guarantees/authoring/paragraphs-unwrap-by-default.md
  it("defaults paragraph unwrapping on and remembers switching it off", () => {
    const { unmount } = render(<SettingsView />);
    const checkbox = screen.getByRole("checkbox", { name: "Unwrap Markdown paragraphs" }) as HTMLInputElement;
    expect(checkbox.checked).toBe(true);
    fireEvent.click(checkbox);
    expect(loadUnwrapParagraphs()).toBe(false);
    unmount();
    render(<SettingsView />);
    expect((screen.getByRole("checkbox", { name: "Unwrap Markdown paragraphs" }) as HTMLInputElement).checked).toBe(false);
  });
  it("lists all five providers with their configured state", async () => {
    render(<SettingsView />);
    for (const label of ["Anthropic", "OpenAI", "OpenRouter", "DeepSeek", "xAI"]) {
      expect(await screen.findByText(label)).toBeTruthy();
    }
    // The configured row shows its masked hint; the rest say so plainly.
    expect(screen.getByText("configured · sk-a…f3")).toBeTruthy();
    expect(screen.getAllByText("not set")).toHaveLength(4);
  });

  it("takes keys through password inputs only — a key is never shown back", async () => {
    render(<SettingsView />);
    const input = await screen.findByLabelText("Anthropic API key");
    expect(input.getAttribute("type")).toBe("password");
    // The masked hint is on screen; nothing resembling a full key can be,
    // because GET does not return one and the view renders only GET's data.
    expect(document.body.textContent).toContain("sk-a…f3");
  });

  it("PUTs only the provider the user changed", async () => {
    render(<SettingsView />);
    const input = await screen.findByLabelText("OpenAI API key");
    fireEvent.change(input, { target: { value: "sk-openai-new" } });
    const row = input.closest("form")!;
    fireEvent.submit(row);

    await waitFor(() => expect(puts).toHaveLength(1));
    // Exactly one field — untouched providers are absent, not re-sent.
    expect(puts[0]).toEqual({ openai: "sk-openai-new" });
    // The row now reads as configured, and the draft is gone from the input.
    expect(await screen.findByText(/configured · sk-o/)).toBeTruthy();
    expect((input as HTMLInputElement).value).toBe("");
  });

  it("clears a key by PUTting null for that provider only", async () => {
    render(<SettingsView />);
    fireEvent.click(await screen.findByRole("button", { name: "Clear" }));
    await waitFor(() => expect(puts).toHaveLength(1));
    expect(puts[0]).toEqual({ anthropic: null });
    expect(await screen.findAllByText("not set")).toHaveLength(5);
  });

  it("says which provider the agent will pick when exactly one key is set", async () => {
    render(<SettingsView />);
    expect(
      await screen.findByText(/the agent uses Anthropic automatically/),
    ).toBeTruthy();
  });
});

</storycode:copy>
<a id="f-5bfac230ae5d-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-5bfac230ae5d-expression_statement-5">
describe("the appearance section", () => {
  it("offers the three themes with dark pressed by default", async () => {
    render(<SettingsView />);
    expect(await screen.findByRole("group", { name: "Theme" })).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "Dark" }).getAttribute("aria-pressed"),
    ).toBe("true");
    for (const label of ["Warm dark", "Light"]) {
      expect(
        screen.getByRole("button", { name: label }).getAttribute("aria-pressed"),
      ).toBe("false");
    }
  });

  it("applies a picked theme immediately and persists it", async () => {
    render(<SettingsView />);
    fireEvent.click(await screen.findByRole("button", { name: "Light" }));
    // Applied on the spot — the page restyles without a reload…
    expect(document.documentElement.dataset.theme).toBe("light");
    // …and persisted under the same key the pre-bundle script reads.
    expect(localStorage.getItem(THEME_KEY)).toBe("light");
    expect(loadTheme()).toBe("light");
    fireEvent.click(screen.getByRole("button", { name: "Warm dark" }));
    expect(document.documentElement.dataset.theme).toBe("warm-dark");
    expect(loadTheme()).toBe("warm-dark");
  });

  it("shows the three display toggles that used to live in the toolbar", async () => {
    render(<SettingsView />);
    expect(await screen.findByRole("group", { name: "Lineage style" })).toBeTruthy();
    expect(screen.getByRole("group", { name: "Tab placement" })).toBeTruthy();
    expect(screen.getByRole("group", { name: "Channel width" })).toBeTruthy();
    for (const label of ["Ribbons", "Braces", "Top", "Side", "Narrow", "Normal", "Wide"]) {
      expect(screen.getByRole("button", { name: label })).toBeTruthy();
    }
  });

  it("reflects the persisted defaults as the pressed choices", async () => {
    render(<SettingsView />);
    const braces = await screen.findByRole("button", { name: "Braces" });
    expect(braces.getAttribute("aria-pressed")).toBe("true");
    expect(
      screen.getByRole("button", { name: "Top" }).getAttribute("aria-pressed"),
    ).toBe("true");
    expect(
      screen.getByRole("button", { name: "Normal" }).getAttribute("aria-pressed"),
    ).toBe("true");
  });

  it("persists each toggle through its lib so the workspace re-reads it on remount", async () => {
    render(<SettingsView />);
    fireEvent.click(await screen.findByRole("button", { name: "Ribbons" }));
    expect(loadRibbonStyle()).toBe("bands");
    fireEvent.click(screen.getByRole("button", { name: "Side" }));
    expect(loadTabStyle()).toBe("side");
    fireEvent.click(screen.getByRole("button", { name: "Wide" }));
    expect(loadChannelWidth()).toBe(72);
    // The keys are unchanged from the toolbar era — stored choices survive.
    expect(localStorage.getItem(RIBBON_STYLE_KEY)).toBe("bands");
    expect(localStorage.getItem(TAB_STYLE_KEY)).toBe("side");
    expect(localStorage.getItem(CHANNEL_WIDTH_KEY)).toBe("72");
  });

  it("offers a lineage-visibility choice, defaulting to the caret", async () => {
    render(<SettingsView />);
    expect(
      await screen.findByRole("group", { name: "Lineage visibility" }),
    ).toBeTruthy();
    expect(
      screen
        .getByRole("button", { name: "With the caret" })
        .getAttribute("aria-pressed"),
    ).toBe("true");
    // The old always-on reading is a setting, not a thing that was removed.
    fireEvent.click(screen.getByRole("button", { name: "Always" }));
    expect(loadRibbonVisibility()).toBe("always");
    expect(localStorage.getItem(RIBBON_VISIBILITY_KEY)).toBe("always");
  });

  it("offers a word-navigation choice, defaulting to whole words", async () => {
    render(<SettingsView />);
    expect(await screen.findByRole("group", { name: "Word navigation" })).toBeTruthy();
    expect(
      screen.getByRole("button", { name: "Whole words" }).getAttribute("aria-pressed"),
    ).toBe("true");
    fireEvent.click(screen.getByRole("button", { name: "Subwords" }));
    expect(loadWordMotion()).toBe("subword");
    expect(localStorage.getItem(WORD_MOTION_KEY)).toBe("subword");
  });

  it("saves a custom window title through PUT /api/settings/ui", async () => {
    render(<SettingsView />);
    const input = await screen.findByLabelText("Window title");
    fireEvent.change(input, { target: { value: "My Lab Notebook" } });
    fireEvent.submit(input.closest("form")!);
    await waitFor(() => expect(uiPuts).toHaveLength(1));
    expect(uiPuts[0]).toEqual({ window_title: "My Lab Notebook" });
    expect(await screen.findByText("custom")).toBeTruthy();
  });

  it("clears the custom title by PUTting null", async () => {
    uiSettings = { window_title: "Old title", format_on_save: false };
    render(<SettingsView />);
    // The saved title arrives in the input, and a Clear button with it. The
    // configured key row has its own Clear, so scope to the title's form.
    const input = await screen.findByLabelText("Window title");
    await waitFor(() => expect((input as HTMLInputElement).value).toBe("Old title"));
    fireEvent.click(within(input.closest("form")!).getByRole("button", { name: "Clear" }));
    await waitFor(() => expect(uiPuts).toHaveLength(1));
    expect(uiPuts[0]).toEqual({ window_title: null });
    expect(await screen.findByText("default (folder name)")).toBeTruthy();
  });

  it("treats a blanked-out field as clearing, not a title of nothing", async () => {
    uiSettings = { window_title: "Old title", format_on_save: false };
    render(<SettingsView />);
    const input = await screen.findByLabelText("Window title");
    await waitFor(() => expect((input as HTMLInputElement).value).toBe("Old title"));
    fireEvent.change(input, { target: { value: "   " } });
    fireEvent.submit(input.closest("form")!);
    await waitFor(() => expect(uiPuts).toHaveLength(1));
    expect(uiPuts[0]).toEqual({ window_title: null });
  });

  // Protects docs/guarantees/editor-intelligence/save-can-format-first.md
  it("turns format-on-save on through PUT /api/settings/ui", async () => {
    render(<SettingsView />);
    const box = (await screen.findByLabelText("Format on save")) as HTMLInputElement;
    expect(box.checked).toBe(false);
    fireEvent.click(box);
    await waitFor(() => expect(uiPuts.at(-1)).toEqual({ format_on_save: true }));
    await waitFor(() => expect(box.checked).toBe(true));
  });

  // Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
  it("keeps saved-file recovery off by default and persists an opt-in", async () => {
    render(<SettingsView />);
    const box = (await screen.findByLabelText("Retain unsaved changes")) as HTMLInputElement;
    expect(box.checked).toBe(false);
    fireEvent.click(box);
    await waitFor(() =>
      expect(uiPuts.at(-1)).toEqual({ retain_unsaved_saved_files: true }),
    );
    expect(box.checked).toBe(true);
  });
});

</storycode:copy>
<a id="f-5bfac230ae5d-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-5bfac230ae5d-expression_statement-6">
it("groups Files experiments together, defaults them off, and saves opt-ins", () => {
  render(<SettingsView />);
  const experiments = within(screen.getByRole("region", { name: "Experiments" }));
  const windows = experiments.getByRole("checkbox", { name: "Show windows in Files pane" }) as HTMLInputElement;
  const parallax = experiments.getByRole("checkbox", { name: "Files pane parallax" }) as HTMLInputElement;
  expect(windows.checked).toBe(false);
  expect(parallax.checked).toBe(false);
  fireEvent.click(windows);
  fireEvent.click(parallax);
  expect(loadFilesWindows()).toBe(true);
  expect(loadFilesParallax()).toBe(true);
});
</storycode:copy>

### Assemble `apps/web/src/views/SettingsView.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/SettingsView.test.tsx"><storycode:paste select="#f-5bfac230ae5d-providers"/><storycode:paste select="#f-5bfac230ae5d-puts"/><storycode:paste select="#f-5bfac230ae5d-uiSettings"/><storycode:paste select="#f-5bfac230ae5d-uiPuts"/><storycode:paste select="#f-5bfac230ae5d-expression_statement"/><storycode:paste select="#f-5bfac230ae5d-expression_statement-2"/><storycode:paste select="#f-5bfac230ae5d-expression_statement-3"/><storycode:paste select="#f-5bfac230ae5d-expression_statement-4"/><storycode:paste select="#f-5bfac230ae5d-expression_statement-5"/><storycode:paste select="#f-5bfac230ae5d-expression_statement-6"/></storycode:file>


</storycode:doc>
