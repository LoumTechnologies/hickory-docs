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
