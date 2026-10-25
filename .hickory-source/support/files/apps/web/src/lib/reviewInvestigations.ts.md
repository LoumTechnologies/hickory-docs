<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/reviewInvestigations.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `545fc07b75474bc365cee45329868ff2f7dc09e0e942558b6565eebd587cb7a5`.

<book:entry path="apps/web/src/lib/reviewInvestigations.ts">
/** Explicit one-recipient registry. Registration never starts an investigation. */
export interface InvestigationConversation {
  id: string;
  label: string;
  send: (prompt: string) => Promise<string>;
}
const conversations = new Map<string, InvestigationConversation>();
const listeners = new Set<() => void>();
let snapshot: InvestigationConversation[] = [];
function changed() { snapshot = [...conversations.values()]; listeners.forEach(listener => listener()); }
export function registerInvestigationConversation(conversation: InvestigationConversation): () => void {
  conversations.set(conversation.id, conversation); changed();
  return () => { if (conversations.get(conversation.id) === conversation) { conversations.delete(conversation.id); changed(); } };
}
export const investigationConversations = {
  subscribe: (listener: () => void) => { listeners.add(listener); return () => { listeners.delete(listener); }; },
  snapshot: () => snapshot,
};
export async function sendInvestigation(id: string, prompt: string): Promise<string> {
  const conversation = conversations.get(id);
  if (!conversation) throw new Error("The selected conversation closed. Select an open agent pane.");
  return conversation.send(prompt);
}
</book:entry>

</book:doc>
