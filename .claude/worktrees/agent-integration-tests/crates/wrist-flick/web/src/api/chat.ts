import type { ChatEvent } from './types'

export async function streamChat(
  conversationId: string,
  content: string,
  onEvent: (event: ChatEvent) => void,
  signal?: AbortSignal,
): Promise<void> {
  const res = await fetch(
    `/api/v1/conversations/${encodeURIComponent(conversationId)}/messages`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content }),
      signal,
    },
  )

  if (!res.ok) {
    throw new Error(`Chat error: ${res.status}`)
  }

  const reader = res.body?.getReader()
  if (!reader) {
    throw new Error('No response body')
  }

  const decoder = new TextDecoder()
  let buffer = ''
  let currentEvent = ''

  for (;;) {
    const { done, value } = await reader.read()
    if (done) break

    buffer += decoder.decode(value, { stream: true })

    const lines = buffer.split('\n')
    // Keep the last potentially incomplete line in the buffer
    buffer = lines.pop() ?? ''

    for (const line of lines) {
      if (line.startsWith(':')) {
        // SSE comment / keep-alive ping — skip
        continue
      }

      if (line.startsWith('event:')) {
        currentEvent = line.slice(6).trim()
        continue
      }

      if (line.startsWith('data:')) {
        const data = line.slice(5).trim()
        if (data && currentEvent) {
          try {
            const parsed = JSON.parse(data) as ChatEvent
            onEvent(parsed)
          } catch {
            // Malformed JSON — skip
          }
        }
        currentEvent = ''
        continue
      }

      // Empty line resets event state
      if (line.trim() === '') {
        currentEvent = ''
      }
    }
  }
}
