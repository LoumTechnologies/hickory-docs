import { useState, type FormEvent } from 'react'

interface Props {
  onSubmit: (key: string, content: string) => void
}

export function CreateMemoryForm({ onSubmit }: Props) {
  const [key, setKey] = useState('')
  const [content, setContent] = useState('')

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault()
    const k = key.trim()
    const c = content.trim()
    if (!k || !c) return
    onSubmit(k, c)
    setKey('')
    setContent('')
  }

  return (
    <form className="create-form" onSubmit={handleSubmit}>
      <input
        type="text"
        placeholder="Key"
        value={key}
        onChange={(e) => setKey(e.target.value)}
      />
      <textarea
        placeholder="Content"
        value={content}
        onChange={(e) => setContent(e.target.value)}
        rows={2}
      />
      <button
        type="submit"
        className="btn-primary"
        disabled={!key.trim() || !content.trim()}
      >
        Add Memory
      </button>
    </form>
  )
}
