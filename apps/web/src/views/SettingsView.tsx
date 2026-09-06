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
// launch). The workspace re-reads all of them when it remounts, which
// navigating back from this page always causes.

import { useEffect, useState } from "react";

import { api } from "../api/client";
import { setFormatOnSave } from "../lib/formatOnSave";
import { KeyboardSection } from "./KeyboardSection";
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

/**
 * The display preferences. The toggles write localStorage on the spot
 * (the same libs the workspace reads at mount); the window title round-trips
 * through GET/PUT /api/settings/ui so the desktop shell can also read it at
 * launch for the native title bar.
 */
function AppearanceSection() {
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

  const applyTitle = (value: string | null) => {
    setTitleError(null);
    api.saveSettingsUi({ window_title: value }).then(
      (ui) => {
        setSavedTitle(ui.window_title);
        setTitleDraft(ui.window_title ?? "");
      },
      (e) => setTitleError(e instanceof Error ? e.message : String(e)),
    );
  };

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

/**
 * Editing behaviour, as opposed to how the app looks.
 *
 * One row so far, and it is here rather than under Appearance because it
 * changes what a key DOES: a person hunting for it after Ctrl+→ overshot is
 * not looking under "Appearance". The keymap re-reads the stored value on
 * every Ctrl+arrow, so the change lands in editors that are already open
 * without navigating back — see editor/wordMotion.ts.
 */
function EditingSection() {
  const [wordMotion, setWordMotion] = useState<WordMotion>(() => loadWordMotion());
  // Format on save, as the server knows it. Off until the read lands: the
  // default, and the honest answer while nothing is known.
  const [formatSave, setFormatSave] = useState(false);
  const [formatError, setFormatError] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    api.settingsUi().then(
      (ui) => {
        if (!live) return;
        setFormatSave(ui.format_on_save === true);
        setFormatOnSave(ui.format_on_save === true);
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

  return (
    <section className="settings__appearance" aria-label="Editing">
      <h2 className="settings__section-title">Editing</h2>
      <div className="settings__rows">
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
            <input
              id="format-on-save"
              type="checkbox"
              checked={formatSave}
              onChange={(event) => applyFormatSave(event.target.checked)}
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
      </div>
      <p className="muted settings__note">
        Subwords is what a code editor calls CamelHumps: <code>Ctrl+→</code> from
        the start of <code>XMLHttpRequest</code> stops at <code>Http</code>, then
        at <code>Request</code>. Shift extends the selection the same way. This
        is the motion you want in code and the wrong one in prose, and a{" "}
        <code>.hick</code> document is both — so it is your choice rather than
        the file’s. On macOS the keys are Option+← and Option+→.
      </p>
    </section>
  );
}

export function SettingsView() {
  const [providers, setProviders] = useState<ProviderKey[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [saved, setSaved] = useState<string | null>(null);

  useEffect(() => {
    document.title = "Settings — Hickory Docs";
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
        navigate("/");
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const configured = (providers ?? []).filter((p) => p.configured);

  return (
    <div className="start settings">
      <div className="start__inner">
        <header className="settings__head">
          <h1 className="start__title">Settings</h1>
          <button
            className="btn"
            onClick={() => navigate("/")}
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
        <KeyboardSection />
      </div>
    </div>
  );
}
