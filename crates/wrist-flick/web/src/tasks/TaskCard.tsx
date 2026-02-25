import type { Task } from '../api/types'

interface Props {
  task: Task
  onToggle: (id: string, currentStatus: string) => void
  onDelete: (id: string) => void
}

export function TaskCard({ task, onToggle, onDelete }: Props) {
  const isDone = task.status === 'done'

  return (
    <div className={`task-card ${isDone ? 'task-done' : ''}`}>
      <button
        className="task-toggle"
        onClick={() => onToggle(task.id, task.status)}
        title={isDone ? 'Mark open' : 'Mark done'}
      >
        {isDone ? '\u2713' : '\u25CB'}
      </button>
      <div className="task-body">
        <span className="task-description">{task.description}</span>
        {task.due_date && (
          <span className="task-due">Due: {task.due_date}</span>
        )}
      </div>
      <button className="btn-danger-sm" onClick={() => onDelete(task.id)}>
        Delete
      </button>
    </div>
  )
}
