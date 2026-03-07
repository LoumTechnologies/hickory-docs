import { useEffect, useState } from 'react'
import { NavLink } from 'react-router-dom'
import type { Conversation } from '../api/types'
import { listConversations } from '../api/conversations'

export function ConversationList() {
  const [conversations, setConversations] = useState<Conversation[]>([])

  useEffect(() => {
    listConversations().then(setConversations).catch(() => {
      // Silently fail — sidebar is non-critical
    })
  }, [])

  // Re-fetch when navigating (conversations may have been created)
  useEffect(() => {
    const interval = setInterval(() => {
      listConversations().then(setConversations).catch(() => {})
    }, 5000)
    return () => clearInterval(interval)
  }, [])

  if (conversations.length === 0) return null

  return (
    <>
      <div className="sidebar-section-label">Conversations</div>
      {conversations.map((c) => (
        <NavLink
          key={c.id}
          to={`/c/${c.id}`}
          className="conversation-item"
          title={c.title}
        >
          {c.title}
        </NavLink>
      ))}
    </>
  )
}
