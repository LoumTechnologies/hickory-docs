import { useReducer, useRef, useCallback } from 'react'
import type { Message, ChatEvent } from '../api/types'
import { streamChat } from '../api/chat'
import { createConversation } from '../api/conversations'

export interface SideEffect {
  type: 'memory_created' | 'task_created'
  id: string
  label: string
}

interface ChatState {
  messages: Message[]
  streaming: boolean
  error: string | null
  sideEffects: SideEffect[]
}

type ChatAction =
  | { type: 'SET_MESSAGES'; messages: Message[] }
  | { type: 'ADD_USER_MESSAGE'; message: Message }
  | { type: 'STREAM_START'; messageId: string }
  | { type: 'STREAM_DELTA'; delta: string }
  | { type: 'STREAM_END' }
  | { type: 'STREAM_ERROR'; message: string }
  | { type: 'SIDE_EFFECT'; effect: SideEffect }
  | { type: 'CLEAR' }

function reducer(state: ChatState, action: ChatAction): ChatState {
  switch (action.type) {
    case 'SET_MESSAGES':
      return { ...state, messages: action.messages, error: null, sideEffects: [] }
    case 'ADD_USER_MESSAGE':
      return { ...state, messages: [...state.messages, action.message] }
    case 'STREAM_START':
      return {
        ...state,
        streaming: true,
        error: null,
        messages: [
          ...state.messages,
          {
            id: action.messageId,
            role: 'assistant',
            content: '',
            created_at: new Date().toISOString(),
          },
        ],
      }
    case 'STREAM_DELTA': {
      const msgs = [...state.messages]
      const last = msgs[msgs.length - 1]
      if (last) {
        msgs[msgs.length - 1] = { ...last, content: last.content + action.delta }
      }
      return { ...state, messages: msgs }
    }
    case 'STREAM_END':
      return { ...state, streaming: false }
    case 'STREAM_ERROR':
      return { ...state, streaming: false, error: action.message }
    case 'SIDE_EFFECT':
      return { ...state, sideEffects: [...state.sideEffects, action.effect] }
    case 'CLEAR':
      return { messages: [], streaming: false, error: null, sideEffects: [] }
  }
}

const initialState: ChatState = {
  messages: [],
  streaming: false,
  error: null,
  sideEffects: [],
}

export function useChat(
  conversationId: string | undefined,
  onConversationCreated: (id: string) => void,
) {
  const [state, dispatch] = useReducer(reducer, initialState)
  const abortRef = useRef<AbortController | null>(null)

  const setMessages = useCallback((messages: Message[]) => {
    dispatch({ type: 'SET_MESSAGES', messages })
  }, [])

  const clear = useCallback(() => {
    dispatch({ type: 'CLEAR' })
  }, [])

  const cancel = useCallback(() => {
    abortRef.current?.abort()
    abortRef.current = null
  }, [])

  const send = useCallback(
    async (content: string) => {
      // Cancel any in-flight stream
      cancel()

      let targetId = conversationId

      // Create conversation if needed
      if (!targetId) {
        try {
          const conv = await createConversation({
            title: content.slice(0, 60),
          })
          targetId = conv.id
          onConversationCreated(targetId)
        } catch {
          dispatch({
            type: 'STREAM_ERROR',
            message: 'Failed to create conversation',
          })
          return
        }
      }

      // Add user message to UI immediately
      dispatch({
        type: 'ADD_USER_MESSAGE',
        message: {
          id: `temp-${Date.now()}`,
          role: 'user',
          content,
          created_at: new Date().toISOString(),
        },
      })

      const controller = new AbortController()
      abortRef.current = controller

      try {
        await streamChat(
          targetId,
          content,
          (event: ChatEvent) => {
            switch (event.type) {
              case 'message_start':
                dispatch({ type: 'STREAM_START', messageId: event.message_id })
                break
              case 'content_delta':
                dispatch({ type: 'STREAM_DELTA', delta: event.delta })
                break
              case 'memory_created':
                dispatch({
                  type: 'SIDE_EFFECT',
                  effect: {
                    type: 'memory_created',
                    id: event.id,
                    label: event.key,
                  },
                })
                break
              case 'task_created':
                dispatch({
                  type: 'SIDE_EFFECT',
                  effect: {
                    type: 'task_created',
                    id: event.id,
                    label: event.description,
                  },
                })
                break
              case 'message_end':
                dispatch({ type: 'STREAM_END' })
                break
              case 'error':
                dispatch({ type: 'STREAM_ERROR', message: event.message })
                break
            }
          },
          controller.signal,
        )
      } catch (err) {
        if (err instanceof DOMException && err.name === 'AbortError') {
          return
        }
        dispatch({
          type: 'STREAM_ERROR',
          message: err instanceof Error ? err.message : 'Stream failed',
        })
      }
    },
    [conversationId, onConversationCreated, cancel],
  )

  return { ...state, send, setMessages, clear, cancel }
}
