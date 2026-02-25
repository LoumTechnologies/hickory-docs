import { useState, type FormEvent } from 'react'

interface Props {
  onSubmit: (description: string) => void
}

export function CreateTaskForm({ onSubmit }: Props) {
  const [description, setDescription] = useState('')

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault()
    const d = description.trim()
    if (!d) return
    onSubmit(d)
    setDescription('')
  }

  return (
    <form className="create-form" onSubmit={handleSubmit}>
      <input
        type="text"
        placeholder="New task description"
        value={description}
        onChange={(e) => setDescription(e.target.value)}
      />
      <button
        type="submit"
        className="btn-primary"
        disabled={!description.trim()}
      >
        Add Task
      </button>
    </form>
  )
}
