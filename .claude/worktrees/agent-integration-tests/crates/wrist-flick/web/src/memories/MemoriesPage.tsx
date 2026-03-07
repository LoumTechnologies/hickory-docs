import { useEffect, useState, useCallback, useRef } from 'react'
import type { Memory, MemorySearchResult } from '../api/types'
import { listMemories, createMemory, deleteMemory, searchMemories } from '../api/memories'
import { MemoryCard } from './MemoryCard'
import { CreateMemoryForm } from './CreateMemoryForm'
import './MemoriesPage.css'

export function MemoriesPage() {
  const [memories, setMemories] = useState<Memory[]>([])
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<MemorySearchResult[] | null>(null)
  const debounceRef = useRef<ReturnType<typeof setTimeout>>()

  const load = useCallback(() => {
    listMemories().then(setMemories).catch(() => {})
  }, [])

  useEffect(() => {
    load()
  }, [load])

  // Debounced search
  useEffect(() => {
    clearTimeout(debounceRef.current)
    if (!searchQuery.trim()) {
      setSearchResults(null)
      return
    }
    debounceRef.current = setTimeout(() => {
      searchMemories(searchQuery.trim())
        .then(setSearchResults)
        .catch(() => setSearchResults([]))
    }, 300)
    return () => clearTimeout(debounceRef.current)
  }, [searchQuery])

  const handleCreate = useCallback(
    async (key: string, content: string) => {
      await createMemory({ key, content })
      load()
    },
    [load],
  )

  const handleDelete = useCallback(
    async (id: string) => {
      await deleteMemory(id)
      load()
      // Re-run search if active
      if (searchQuery.trim()) {
        searchMemories(searchQuery.trim())
          .then(setSearchResults)
          .catch(() => {})
      }
    },
    [load, searchQuery],
  )

  const displayItems = searchResults ?? memories

  return (
    <div className="memories-page">
      <h2 className="page-title">Memories</h2>
      <CreateMemoryForm onSubmit={handleCreate} />
      <input
        type="text"
        className="search-input"
        placeholder="Search memories..."
        value={searchQuery}
        onChange={(e) => setSearchQuery(e.target.value)}
      />
      <div className="card-list">
        {displayItems.length === 0 ? (
          <p className="empty-text">
            {searchQuery.trim() ? 'No results found.' : 'No memories yet.'}
          </p>
        ) : (
          displayItems.map((m) => (
            <MemoryCard key={m.id} memory={m} onDelete={handleDelete} />
          ))
        )}
      </div>
    </div>
  )
}
