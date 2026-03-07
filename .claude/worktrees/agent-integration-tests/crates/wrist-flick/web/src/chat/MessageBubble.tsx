import type { Message } from '../api/types'

interface Props {
  message: Message
}

export function MessageBubble({ message }: Props) {
  const isUser = message.role === 'user'

  return (
    <div className={`message-bubble ${isUser ? 'message-user' : 'message-assistant'}`}>
      <div className="message-content">
        {message.content || (
          <span className="message-placeholder">&hellip;</span>
        )}
      </div>
    </div>
  )
}
