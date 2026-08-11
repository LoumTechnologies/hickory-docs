import { useEffect, useState } from "react";
import { api } from "../api/client";
import type { LlmKeysResponse, User } from "../api/types";
import { navigate } from "../router";

/** The vendors the agent can run on, in the order they are worth trying.
 * Labels are what the vendor calls itself; the value is what the API takes. */
const PROVIDERS = [
  { value: "anthropic", label: "Anthropic (Claude)", hint: "sk-ant-…" },
  { value: "openai", label: "OpenAI", hint: "sk-…" },
  { value: "grok", label: "xAI (Grok)", hint: "xai-…" },
  { value: "deepseek", label: "DeepSeek", hint: "sk-…" },
];

/** Vendor name for a stored row. The API stores xAI's canonical name, which
 * is not the name on the button that saved it. */
function providerLabel(value: string): string {
  if (value === "xai") return "xAI (Grok)";
  return PROVIDERS.find((p) => p.value === value)?.label ?? value;
}

/**
 * Account settings: the provider API keys this account brings.
 *
 * The screen's job is to make one thing unambiguous — whose key an agent run
 * spends. On a `byo_key` plan there is no fallback to ours, so an empty state
 * here is the reason the agent will not run, and it says so rather than
 * leaving the user to discover it from a 402 later.
 */
export function SettingsView({ user }: { user: User }) {
  const [state, setState] = useState<LlmKeysResponse | null>(null);
  const [provider, setProvider] = useState("anthropic");
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState<string | null>(null);

  useEffect(() => {
    api.llmKeys().then(setState, (e) => setError(String(e.message ?? e)));
  }, []);

  const save = async (e: React.FormEvent) => {
    e.preventDefault();
    const key = apiKey.trim();
    if (!key) return;
    setBusy(true);
    setError(null);
    setSaved(null);
    try {
      await api.saveLlmKey(provider, key, { model: model.trim() || undefined });
      setState(await api.llmKeys());
      // Clear the field on success: the value is stored and this input has no
      // further use for it, and a key left sitting in a form is a key sitting
      // in a screenshot.
      setApiKey("");
      setModel("");
      setSaved(providerLabel(provider));
    } catch (err: any) {
      setError(String(err.message ?? err));
    } finally {
      setBusy(false);
    }
  };

  const remove = async (p: string) => {
    setBusy(true);
    setError(null);
    try {
      setState(await api.deleteLlmKey(p));
    } catch (err: any) {
      setError(String(err.message ?? err));
    } finally {
      setBusy(false);
    }
  };

  const select = async (p: string) => {
    setBusy(true);
    setError(null);
    try {
      setState(await api.selectLlmKey(p));
    } catch (err: any) {
      setError(String(err.message ?? err));
    } finally {
      setBusy(false);
    }
  };

  const keys = state?.keys ?? [];
  const byoPlan = state?.plan_agent === "byo_key";
  const noneActive = keys.length > 1 && !keys.some((k) => k.active);

  return (
    <div className="settings-page">
      <h1>Settings</h1>
      <p className="muted">{user.email}</p>

      <section className="settings-section">
        <h2>API keys</h2>
        <p className="muted">
          {byoPlan ? (
            <>
              Your plan runs the AI agent on <strong>your own</strong> provider
              key. Keys are encrypted before they are stored, and this page can
              never show one again — only its last four characters.
            </>
          ) : (
            <>
              Your plan includes an agent allowance. Storing a key here runs
              the agent on <strong>your</strong> account instead, leaving the
              allowance unspent.
            </>
          )}
        </p>

        {error && <div className="banner banner-fail">{error}</div>}
        {saved && (
          <div className="banner banner-pass">
            {saved} key saved and accepted by the provider.
          </div>
        )}

        {state && !state.storage_available && (
          <div className="banner banner-warn">
            This deployment cannot store API keys yet — its operator has not
            set <code>KEY_ENCRYPTION_KEY</code>. Nothing you can do from here;
            the fix is one command on the server (<code>just gen-key</code>).
          </div>
        )}

        {byoPlan && state?.storage_available && keys.length === 0 && (
          <div className="banner banner-warn">
            No key stored, so the agent cannot run. Add one below — it takes a
            minute and the first request proves it works.
          </div>
        )}

        {noneActive && (
          <div className="banner banner-warn">
            You have {keys.length} keys and none is selected. Choose which one
            agent runs should use.
          </div>
        )}

        {keys.length > 0 && (
          <table className="settings-keys">
            <thead>
              <tr>
                <th>Provider</th>
                <th>Key</th>
                <th>Model</th>
                <th>Last used</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {keys.map((k) => (
                <tr key={k.provider} className={k.active ? "is-active" : undefined}>
                  <td>
                    {providerLabel(k.provider)}
                    {k.active && <span className="chip">in use</span>}
                  </td>
                  <td className="mono">••••{k.last4}</td>
                  <td className="mono muted">{k.model ?? "default"}</td>
                  <td className="muted">
                    {k.last_used_at
                      ? new Date(k.last_used_at).toLocaleDateString()
                      : "never"}
                  </td>
                  <td className="settings-key-actions">
                    {!k.active && keys.length > 1 && (
                      <button
                        className="btn btn-sm"
                        disabled={busy}
                        onClick={() => select(k.provider)}
                      >
                        Use this
                      </button>
                    )}
                    <button
                      className="btn btn-sm btn-link"
                      disabled={busy}
                      onClick={() => remove(k.provider)}
                    >
                      Remove
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}

        {state?.storage_available && (
          <form onSubmit={save} className="settings-key-form">
            <label>
              Provider
              <select
                value={provider}
                onChange={(e) => setProvider(e.target.value)}
              >
                {PROVIDERS.map((p) => (
                  <option key={p.value} value={p.value}>
                    {p.label}
                  </option>
                ))}
              </select>
            </label>
            <label>
              API key
              <input
                type="password"
                autoComplete="off"
                spellCheck={false}
                placeholder={
                  PROVIDERS.find((p) => p.value === provider)?.hint ?? ""
                }
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                required
              />
            </label>
            <label>
              Model <span className="muted">(optional)</span>
              <input
                type="text"
                spellCheck={false}
                placeholder="provider default"
                value={model}
                onChange={(e) => setModel(e.target.value)}
              />
            </label>
            <button className="btn btn-primary" type="submit" disabled={busy}>
              {busy ? "Checking with the provider…" : "Save key"}
            </button>
          </form>
        )}
      </section>

      <button className="btn btn-link" onClick={() => navigate("/projects")}>
        Back to projects
      </button>
    </div>
  );
}
