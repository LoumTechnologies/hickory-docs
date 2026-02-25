import { useEffect, useCallback } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useChat } from './useChat'
import { MessageList } from './MessageList'
import { ChatInput } from './ChatInput'
import { getConversation } from '../api/conversations'
import type { SideEffect } from './useChat'
import './ChatPage.css'

export function ChatPage() {
  const { conversationId } = useParams<{ conversationId: string }>()
  const navigate = useNavigate()

  const onConversationCreated = useCallback(
    (id: string) => {
      navigate(`/c/${id}`, { replace: true })
    },
    [navigate],
  )

  const {
    messages,
    streaming,
    error,
    sideEffects,
    send,
    setMessages,
    clear,
    cancel,
  } = useChat(conversationId, onConversationCreated)

  // Load messages when conversationId changes
  useEffect(() => {
    cancel()
    if (conversationId) {
      getConversation(conversationId)
        .then((detail) => setMessages(detail.messages))
        .catch(() => setMessages([]))
    } else {
      clear()
    }
  }, [conversationId, setMessages, clear, cancel])

  return (
    <div className="chat-page">
      <MessageList messages={messages} streaming={streaming} />
      {sideEffects.length > 0 && (
        <SideEffectBar effects={sideEffects} />
      )}
      {error && <div className="chat-error">{error}</div>}
      <ChatInput onSend={send} disabled={streaming} />
    </div>
  )
}

function SideEffectBar({ effects }: { effects: SideEffect[] }) {
  return (
    <div className="side-effects">
      {effects.map((e) => (
        <span key={e.id} className="side-effect-chip">
          {e.type === 'memory_created' ? 'Memory' : 'Task'}: {e.label}
        </span>
      ))}
    </div>
  )
}
