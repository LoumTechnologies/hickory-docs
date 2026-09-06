// Settings → Keyboard: the profile, and every action's binding.
//
// A profile is a starting point, not a mode: picking JetBrains binds every
// action the JetBrains way, and any single one can then be changed or
// unbound, which is recorded as an override on top of the profile. "Reset"
// on a row drops its override; "Reset all" drops them all.
//
// Menu-bar actions are the shell's, built at launch, so a change to one of
// them is said to land at the next launch rather than pretended live. See
// docs/guarantees/editor-intelligence/every-shortcut-is-a-setting.md.
import { useEffect, useMemo, useState } from "react";
import type { KeyboardEvent } from "react";

import {
  ACTIONS,
  PROFILES,
  conflicts,
  formatBinding,
  isMac,
  keymapSettings,
  onKeymapChange,
  parseBinding,
  profileBinding,
  resolve,
  saveKeymap,
} from "../lib/keymap";
import type { ActionSpec, KeymapSettings, Profile, Scope } from "../lib/keymap";

const SCOPES: { id: Scope; title: string; note?: string }[] = [
  { id: "editor", title: "Editor" },
  { id: "workspace", title: "Workspace" },
  { id: "tree", title: "File tree (dired)", note: "With a tree row focused." },
  { id: "menu", title: "Menu bar", note: "Takes effect at the next launch of the desktop app." },
];

/** A keydown as a binding string, or null for a bare modifier. */
export function bindingFromEvent(event: KeyboardEvent, mac = isMac()): string | null {
  const key = event.key;
  if (["Control", "Shift", "Alt", "Meta", "OS"].includes(key)) return null;
  const parts: string[] = [];
  if (mac ? event.metaKey : event.ctrlKey) parts.push("Mod");
  if (mac && event.ctrlKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  parts.push(key === " " ? "Space" : key.length === 1 ? key.toUpperCase() : key);
  return parts.join("+");
}

export function KeyboardSection() {
  const [settings, setSettings] = useState<KeymapSettings>(() => keymapSettings());
  const [error, setError] = useState<string | null>(null);
  /** The action whose next keystroke is being recorded, and the first
   * stroke of a chord if one has been typed. */
  const [recording, setRecording] = useState<{ id: string; first: string | null } | null>(null);
  useEffect(() => onKeymapChange(() => setSettings(keymapSettings())), []);

  const bindings = useMemo(() => resolve(settings), [settings]);
  const clashes = useMemo(() => conflicts(bindings), [bindings]);
  const clashing = new Set(clashes.flat());

  const save = (next: KeymapSettings) => {
    setError(null);
    setSettings(next);
    saveKeymap(next).catch((e: unknown) => setError(e instanceof Error ? e.message : String(e)));
  };
  const setProfile = (profile: Profile) => save({ profile, overrides: settings.overrides });
  const override = (id: string, binding: string) =>
    save({ ...settings, overrides: { ...settings.overrides, [id]: binding } });
  const reset = (id: string) => {
    const overrides = { ...settings.overrides };
    delete overrides[id];
    save({ ...settings, overrides });
  };

  const onRecordKey = (spec: ActionSpec, event: KeyboardEvent) => {
    event.preventDefault();
    event.stopPropagation();
    if (event.key === "Escape") {
      setRecording(null);
      return;
    }
    const stroke = bindingFromEvent(event);
    if (!stroke) return;
    if (recording?.first) {
      override(spec.id, `${recording.first} ${stroke}`);
      setRecording(null);
      return;
    }
    // A menu-bar action cannot carry a chord; anything else may, and a
    // second stroke within the moment makes one.
    if (spec.scope === "menu") {
      override(spec.id, stroke);
      setRecording(null);
      return;
    }
    setRecording({ id: spec.id, first: stroke });
    window.setTimeout(() => {
      setRecording((current) => {
        if (current?.id === spec.id && current.first === stroke) {
          override(spec.id, stroke);
          return null;
        }
        return current;
      });
    }, 900);
  };

  return (
    <section className="settings__appearance" aria-label="Keyboard">
      <h2 className="settings__section-title">Keyboard</h2>
      <div className="settings__rows">
        <div className="settings-row settings-row--appearance">
          <div className="settings-row__who">
            <label className="settings-row__label" htmlFor="keymap-profile">
              Profile
            </label>
            <p className="settings-row__hint">
              A starting point. Change any single key below; a changed key stays yours when the
              profile changes.
            </p>
          </div>
          <div className="settings-row__actions">
            <select
              id="keymap-profile"
              value={settings.profile}
              onChange={(event) => setProfile(event.target.value as Profile)}
            >
              {PROFILES.map((profile) => (
                <option key={profile.id} value={profile.id}>
                  {profile.label}
                </option>
              ))}
            </select>
            <button
              type="button"
              disabled={Object.keys(settings.overrides).length === 0}
              onClick={() => save({ profile: settings.profile, overrides: {} })}
            >
              Reset all
            </button>
          </div>
        </div>
        {error && <p className="error">{error}</p>}
        {clashes.length > 0 && (
          <p className="settings__warning" role="status">
            Two actions share a key:{" "}
            {clashes.map(([a, b]) => `${label(a)} and ${label(b)}`).join("; ")}. The first wins.
          </p>
        )}
        {SCOPES.map((scope) => (
          <div key={scope.id} className="keymap-scope">
            <h3 className="keymap-scope__title">
              {scope.title}
              {scope.note && <span className="keymap-scope__note"> — {scope.note}</span>}
            </h3>
            <table className="keymap-table">
              <tbody>
                {ACTIONS.filter((spec) => spec.scope === scope.id).map((spec) => {
                  const binding = bindings.get(spec.id) ?? "";
                  const overridden = spec.id in settings.overrides;
                  const isRecording = recording?.id === spec.id;
                  return (
                    <tr key={spec.id} className={clashing.has(spec.id) ? "keymap-row--clash" : undefined}>
                      <td className="keymap-table__label">{spec.label}</td>
                      <td className="keymap-table__key">
                        <button
                          type="button"
                          className={`keymap-key${isRecording ? " keymap-key--recording" : ""}`}
                          aria-label={`Change key for ${spec.label}`}
                          onClick={() => setRecording({ id: spec.id, first: null })}
                          onKeyDown={(event) => isRecording && onRecordKey(spec, event)}
                          onBlur={() => isRecording && setRecording(null)}
                        >
                          {isRecording
                            ? recording?.first
                              ? `${formatBinding(recording.first)} …`
                              : "Press keys"
                            : binding
                              ? formatBinding(binding)
                              : "Unbound"}
                        </button>
                      </td>
                      <td className="keymap-table__actions">
                        {binding && !isRecording && (
                          <button type="button" onClick={() => override(spec.id, "")}>
                            Unbind
                          </button>
                        )}
                        {overridden && (
                          <button
                            type="button"
                            onClick={() => reset(spec.id)}
                            data-tip={`Back to ${profileBinding(spec, settings.profile) || "unbound"}`}
                          >
                            Reset
                          </button>
                        )}
                        {binding && !parseBinding(binding) && <span className="error">unreadable</span>}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        ))}
      </div>
    </section>
  );
}

function label(id: string): string {
  return ACTIONS.find((spec) => spec.id === id)?.label ?? id;
}
