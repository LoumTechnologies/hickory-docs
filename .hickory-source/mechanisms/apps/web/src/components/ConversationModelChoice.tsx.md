<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/ConversationModelChoice.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6682e6db3454"></a>
## `apps/web/src/components/ConversationModelChoice.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-6682e6db3454-ConversationModelChoice"></a>
### `ConversationModelChoice`

<storycode:copy id="f-6682e6db3454-ConversationModelChoice">
import { useId } from "react";
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
    <input aria-label="Model" value={model} disabled={disabled} placeholder={defaultModel || "default model"} data-tip="Model id for the next turn — leave empty for the provider's default" onChange={e => onModel(e.target.value)} />
  </span>;
}
</storycode:copy>

### Assemble `apps/web/src/components/ConversationModelChoice.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ConversationModelChoice.tsx"><storycode:paste select="#f-6682e6db3454-ConversationModelChoice"/></storycode:file>

</storycode:doc>
