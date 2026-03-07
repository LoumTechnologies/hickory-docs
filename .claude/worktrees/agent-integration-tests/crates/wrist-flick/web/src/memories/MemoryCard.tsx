import type { Memory } from '../api/types'

interface Props {
  memory: Memory
  onDelete: (id: string) => void
}

export function MemoryCard({ memory, onDelete }: Props) {
  return (
    <div className="memory-card">
      <div className="memory-card-header">
        <span className="memory-key">{memory.key}</span>
        <button
          className="btn-danger-sm"
          onClick={() => onDelete(memory.id)}
        >
          Delete
        </button>
      </div>
      <p className="memory-content">{memory.content}</p>
    </div>
  )
}
