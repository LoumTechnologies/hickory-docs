import { apiFetch } from './client'
import type {
  Conversation,
  ConversationDetail,
  CreateConversationRequest,
  Message,
} from './types'

export function listConversations(): Promise<Conversation[]> {
  return apiFetch('/api/v1/conversations')
}

export function getConversation(id: string): Promise<ConversationDetail> {
  return apiFetch(`/api/v1/conversations/${encodeURIComponent(id)}`)
}

export function createConversation(
  req: CreateConversationRequest,
): Promise<Conversation> {
  return apiFetch('/api/v1/conversations', {
    method: 'POST',
    body: JSON.stringify(req),
  })
}

export function deleteConversation(id: string): Promise<void> {
  return apiFetch(`/api/v1/conversations/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  })
}

export function listMessages(conversationId: string): Promise<Message[]> {
  return apiFetch(
    `/api/v1/conversations/${encodeURIComponent(conversationId)}/messages`,
  )
}
