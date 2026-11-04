import { useEffect, useId, useState } from "react";
import { request } from "../api/client";
import type { UiOutcome } from "../lib/uiActions";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";

/** Mounted only in the feature's actual settings disclosure. */
export function ConversationModelChoice({ scope, provider, model, disabled, providers, defaultModel, onProvider, onModel }: {
  scope: string; provider: string; model: string; disabled: boolean;
  providers: { id: string; label: string }[]; defaultModel: string;
  onProvider: (provider: string) => void; onModel: (model: string) => void;
}) {
  const parent = `conversation-model:${useId()}`;
  const revision = useUiRevision(() => [scope, provider, model, disabled, providers]);
  useUiCommands([
    { key: `${parent}:provider`, command: { label: "Set conversation provider", parent, subject: parent, parameters: { provider: { type: "string", maxLength: 4096, choices: [...new Set([provider, ...providers.map(p => p.id)])] } }, prerequisites: [], read: () => ({ revision: revision(), enabled: !disabled, value: provider }), execute: async args => { onProvider(args.provider as string); return { status: "completed", value: { provider_requested: args.provider, scope: "Next turn; no request or login performed" } }; } } },
    { key: `${parent}:model`, command: { label: "Set conversation model", parent, subject: parent, parameters: { model: { type: "string", maxLength: 4096 } }, prerequisites: [], read: () => ({ revision: revision(), enabled: !disabled, value: model }), execute: async args => { onModel(args.model as string); return { status: "completed", value: { model_requested: args.model, scope: "Next turn; empty uses provider default" } }; } } },
  ]);
  return <span className="chat-model" role="group" aria-label="Model choice">
    <select aria-label="Provider" value={provider} disabled={disabled} onChange={e => onProvider(e.target.value)}>
      {provider === "" && <option value="">provider…</option>}
      {provider !== "" && !providers.some(p => p.id === provider) && <option value={provider}>{provider}</option>}
      {providers.map(p => <option key={p.id} value={p.id}>{p.label}</option>)}
    </select>
    {(provider === "codex-subscription" || provider === "grok-subscription") && <SubscriptionSignIn key={provider} provider={provider} disabled={disabled} />}
    <input aria-label="Model" value={model} disabled={disabled} placeholder={defaultModel || "default model"} data-tip="Model id for the next turn — leave empty for the provider's default" onChange={e => onModel(e.target.value)} />
  </span>;
}
type SubscriptionStatus = { ready: boolean; auth_url?: string; error?: string | null; login_pending?: boolean };
function SubscriptionSignIn({ provider, disabled }: { provider: string; disabled: boolean }) {
  const codex = provider === "codex-subscription";
  const name = codex ? "Codex" : "Grok";
  const account = codex ? "ChatGPT" : "Grok";
  const endpoint = `/api/agents/${provider}`;
  const parent = `subscription-auth:${useId()}`;
  const [status, setStatus] = useState<SubscriptionStatus | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const revision = useUiRevision(() => [status, error, busy, disabled]);
  useEffect(() => {
    let current = true;
    request<SubscriptionStatus>("GET", endpoint).then(value => {
      if (current) setStatus(value);
    }).catch(reason => { if (current) setError(String(reason)); });
    return () => { current = false; };
  }, [endpoint]);
  const invoke = useUiCommands([false, true].map(login => ({ key: `${parent}:${login}`, command: {
    label: login ? `Sign in to ${name} with ${account}` : `Check ${name} sign-in`, parent, subject: parent,
    parameters: {}, prerequisites: login ? [`The human completes ${account} sign-in`] : [],
    read: () => ({ revision: revision(), enabled: !busy && !disabled }),
    execute: async (_args, actor): Promise<UiOutcome> => {
      if (login && actor.kind === "agent") return { status: "requires_input", request: parent, reason: `A human must sign in with ${account}` };
      setBusy(true); setError("");
      try {
        const result = await request<SubscriptionStatus>(login ? "POST" : "GET", endpoint);
        setStatus(result);
        return { status: "completed", value: { ready: result.ready, billing: codex ? "chatgpt_subscription" : "grok_subscription" } };
      } catch (reason) { setError(String(reason)); return { status: "unavailable", reason: String(reason) }; }
      finally { setBusy(false); }
    },
  } })));
  return <span role="group" aria-label={`${name} subscription sign-in`}>
    <span>{status?.ready ? `${account} sign-in ready` : `Sign in with ${account} to use your ${name} allowance and available credits.`}</span>
    <span>Subscription usage; API charges, credit costs and remaining balance are not estimated. No API fallback or automatic credit purchases.</span>
    {(error || status?.error) && <span role="alert">{error || status?.error} Check your connection and {name} installation, then retry.</span>}
    <button disabled={disabled || busy} onClick={() => void invoke(`${parent}:true`, {})}>{status?.ready ? `Sign in again with ${account}` : `Sign in with ${account}`}</button>
    {status?.login_pending && <span>Complete Grok sign-in in your browser, then Check sign-in. If no browser opens, run <code>grok login</code> in a terminal.</span>}
    {status?.auth_url && <a href={status.auth_url} target="_blank" rel="noreferrer">Continue {account} sign-in</a>}
    <button disabled={disabled || busy} onClick={() => void invoke(`${parent}:false`, {})}>Check sign-in</button>
  </span>;
}
