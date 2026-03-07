import { useEffect, useState, useCallback } from 'react'
import type { Task } from '../api/types'
import { listTasks, createTask, updateTask, deleteTask } from '../api/tasks'
import { TaskCard } from './TaskCard'
import { CreateTaskForm } from './CreateTaskForm'
import './TasksPage.css'

export function TasksPage() {
  const [tasks, setTasks] = useState<Task[]>([])

  const load = useCallback(() => {
    listTasks().then(setTasks).catch(() => {})
  }, [])

  useEffect(() => {
    load()
  }, [load])

  const handleCreate = useCallback(
    async (description: string) => {
      await createTask({ description })
      load()
    },
    [load],
  )

  const handleToggle = useCallback(
    async (id: string, currentStatus: string) => {
      const newStatus = currentStatus === 'done' ? 'open' : 'done'
      await updateTask(id, { status: newStatus })
      load()
    },
    [load],
  )

  const handleDelete = useCallback(
    async (id: string) => {
      await deleteTask(id)
      load()
    },
    [load],
  )

  return (
    <div className="tasks-page">
      <h2 className="page-title">Tasks</h2>
      <CreateTaskForm onSubmit={handleCreate} />
      <div className="card-list">
        {tasks.length === 0 ? (
          <p className="empty-text">No tasks yet.</p>
        ) : (
          tasks.map((t) => (
            <TaskCard
              key={t.id}
              task={t}
              onToggle={handleToggle}
              onDelete={handleDelete}
            />
          ))
        )}
      </div>
    </div>
  )
}
