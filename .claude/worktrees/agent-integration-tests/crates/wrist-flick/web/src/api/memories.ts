import { apiFetch } from './client'
import type { Memory, CreateMemoryRequest, MemorySearchResult } from './types'

export function listMemories(): Promise<Memory[]> {
  return apiFetch('/api/v1/memories')
}

export function createMemory(req: CreateMemoryRequest): Promise<Memory> {
  return apiFetch('/api/v1/memories', {
    method: 'POST',
    body: JSON.stringify(req),
  })
}

export function deleteMemory(id: string): Promise<void> {
  return apiFetch(`/api/v1/memories/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  })
}

export function searchMemories(q: string): Promise<MemorySearchResult[]> {
  return apiFetch(
    `/api/v1/memories/search?q=${encodeURIComponent(q)}`,
  )
}
