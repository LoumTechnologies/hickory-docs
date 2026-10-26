<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/reviewInvestigations.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4b6f1426b118"></a>
## `apps/web/src/lib/reviewInvestigations.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-4b6f1426b118-InvestigationConversation"></a>
### `InvestigationConversation`

Explicit one-recipient registry. Registration never starts an investigation.

<storycode:copy id="f-4b6f1426b118-InvestigationConversation">
/** Explicit one-recipient registry. Registration never starts an investigation. */
export interface InvestigationConversation {
  id: string;
  label: string;
  send: (prompt: string) => Promise<string>;
}
</storycode:copy>
<a id="f-4b6f1426b118-conversations"></a>
### `conversations`

<storycode:copy id="f-4b6f1426b118-conversations">
const conversations = new Map<string, InvestigationConversation>();
</storycode:copy>
<a id="f-4b6f1426b118-listeners"></a>
### `listeners`

<storycode:copy id="f-4b6f1426b118-listeners">
const listeners = new Set<() => void>();
</storycode:copy>
<a id="f-4b6f1426b118-snapshot"></a>
### `snapshot`

<storycode:copy id="f-4b6f1426b118-snapshot">
let snapshot: InvestigationConversation[] = [];
</storycode:copy>
<a id="f-4b6f1426b118-changed"></a>
### `changed`

<storycode:copy id="f-4b6f1426b118-changed">
function changed() { snapshot = [...conversations.values()]; listeners.forEach(listener => listener()); }
</storycode:copy>
<a id="f-4b6f1426b118-registerInvestigationConversation"></a>
### `registerInvestigationConversation`

<storycode:copy id="f-4b6f1426b118-registerInvestigationConversation">
export function registerInvestigationConversation(conversation: InvestigationConversation): () => void {
  conversations.set(conversation.id, conversation); changed();
  return () => { if (conversations.get(conversation.id) === conversation) { conversations.delete(conversation.id); changed(); } };
}
</storycode:copy>
<a id="f-4b6f1426b118-investigationConversations"></a>
### `investigationConversations`

<storycode:copy id="f-4b6f1426b118-investigationConversations">
export const investigationConversations = {
  subscribe: (listener: () => void) => { listeners.add(listener); return () => { listeners.delete(listener); }; },
  snapshot: () => snapshot,
};
</storycode:copy>
<a id="f-4b6f1426b118-sendInvestigation"></a>
### `sendInvestigation`

<storycode:copy id="f-4b6f1426b118-sendInvestigation">
export async function sendInvestigation(id: string, prompt: string): Promise<string> {
  const conversation = conversations.get(id);
  if (!conversation) throw new Error("The selected conversation closed. Select an open agent pane.");
  return conversation.send(prompt);
}
</storycode:copy>

### Assemble `apps/web/src/lib/reviewInvestigations.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/reviewInvestigations.ts"><storycode:paste select="#f-4b6f1426b118-InvestigationConversation"/><storycode:paste select="#f-4b6f1426b118-conversations"/><storycode:paste select="#f-4b6f1426b118-listeners"/><storycode:paste select="#f-4b6f1426b118-snapshot"/><storycode:paste select="#f-4b6f1426b118-changed"/><storycode:paste select="#f-4b6f1426b118-registerInvestigationConversation"/><storycode:paste select="#f-4b6f1426b118-investigationConversations"/><storycode:paste select="#f-4b6f1426b118-sendInvestigation"/></storycode:file>

</storycode:doc>
