// Conversation
export interface Conversation {
  id: string
  title: string
  created_at: string
  updated_at: string
}

export interface ConversationDetail extends Conversation {
  messages: Message[]
}

export interface CreateConversationRequest {
  title?: string
}

// Message
export interface Message {
  id: string
  role: string
  content: string
  created_at: string
}

export interface SendMessageRequest {
  content: string
}

// Memory
export interface Memory {
  id: string
  key: string
  content: string
}

export interface CreateMemoryRequest {
  key: string
  content: string
}

export interface MemorySearchResult {
  id: string
  key: string
  content: string
  rank: number
}

// Task
export interface Task {
  id: string
  description: string
  status: string
  due_date: string | null
}

export interface CreateTaskRequest {
  description: string
}

export interface UpdateTaskRequest {
  description?: string
  status?: string
  due_date?: string
}

// Chat SSE events
export type ChatEvent =
  | { type: 'message_start'; message_id: string; role: string }
  | { type: 'content_delta'; delta: string }
  | { type: 'memory_created'; id: string; key: string }
  | { type: 'task_created'; id: string; description: string }
  | { type: 'message_end'; message_id: string; finish_reason: string }
  | { type: 'error'; message: string }
