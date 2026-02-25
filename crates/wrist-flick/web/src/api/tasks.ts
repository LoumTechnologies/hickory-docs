import { apiFetch } from './client'
import type { Task, CreateTaskRequest, UpdateTaskRequest } from './types'

export function listTasks(): Promise<Task[]> {
  return apiFetch('/api/v1/tasks')
}

export function createTask(req: CreateTaskRequest): Promise<Task> {
  return apiFetch('/api/v1/tasks', {
    method: 'POST',
    body: JSON.stringify(req),
  })
}

export function updateTask(id: string, req: UpdateTaskRequest): Promise<Task> {
  return apiFetch(`/api/v1/tasks/${encodeURIComponent(id)}`, {
    method: 'PATCH',
    body: JSON.stringify(req),
  })
}

export function deleteTask(id: string): Promise<void> {
  return apiFetch(`/api/v1/tasks/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  })
}
