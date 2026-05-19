import { useMemo, useState } from 'react'
import { ConversationCanvas } from './conversation/ConversationCanvas'
import { useAgentSocket } from './useAgentSocket'
import type { UiEvent } from './types'

export default function App() {
  const { events, send, connected, sessionStartedObservedAt } = useAgentSocket()
  const [draft, setDraft] = useState('')
  const [sidebarOpen, setSidebarOpen] = useState(true)
  const [blockingInputLocked, setBlockingInputLocked] = useState(false)

  const relaySessionId = useMemo(() => {
    const m = window.location.pathname.match(/^\/session\/([^/]+)\/?$/)
    return m ? m[1] : null
  }, [])

  const sessionFromEvents = useMemo(() => {
    let latest: Extract<UiEvent, { type: 'session_started' }> | null = null
    for (const ev of events) {
      if (ev.type === 'session_started') latest = ev
    }
    return latest
  }, [events])

  return (
    <div className="flex h-screen min-h-0 flex-col bg-slate-950 text-slate-100">
      {!connected && (
        <div
          role="status"
          className="border-b border-amber-500/40 bg-amber-950/60 px-4 py-2 text-sm text-amber-100"
        >
          Not connected to the agent. Start the agent with the web UI enabled, or check that
          the WebSocket URL is correct
          {import.meta.env.DEV ? ' (dev: requests to /ws are proxied to localhost:7890).' : '.'}
        </div>
      )}

      <div className="flex min-h-0 flex-1">
        {/* Sidebar */}
        <aside
          className={`flex shrink-0 flex-col border-r border-slate-800 bg-slate-900/80 transition-[width] duration-200 ease-out ${
            sidebarOpen ? 'w-72 px-3 py-3' : 'w-11 overflow-hidden px-1 py-2'
          }`}
        >
          <button
            type="button"
            className="mb-2 rounded border border-slate-700 bg-slate-800 px-2 py-1 text-left text-xs text-slate-300 hover:bg-slate-700"
            onClick={() => setSidebarOpen((o) => !o)}
            title={sidebarOpen ? 'Collapse sidebar' : 'Expand sidebar'}
          >
            {sidebarOpen ? '◀' : '▶'}
          </button>
          {sidebarOpen && (
            <div className="space-y-3 text-sm">
              <h2 className="text-xs font-semibold uppercase tracking-wide text-slate-400">
                Session
              </h2>
              {relaySessionId && (
                <p className="rounded bg-indigo-950/50 px-2 py-1 text-xs text-indigo-200">
                  Remote relay session
                </p>
              )}
              <dl className="space-y-2 text-slate-200">
                <div>
                  <dt className="text-slate-500">Model</dt>
                  <dd className="break-all font-mono text-xs">
                    {sessionFromEvents?.model ?? '—'}
                  </dd>
                </div>
                <div>
                  <dt className="text-slate-500">Provider</dt>
                  <dd className="break-all font-mono text-xs">
                    {sessionFromEvents?.provider_info ?? '—'}
                  </dd>
                </div>
                <div>
                  <dt className="text-slate-500">Sandbox image</dt>
                  <dd className="break-all font-mono text-xs">
                    {sessionFromEvents?.sandbox_image ?? '—'}
                  </dd>
                </div>
                <div>
                  <dt className="text-slate-500">Session ID</dt>
                  <dd className="break-all font-mono text-xs">
                    {relaySessionId ??
                      (sessionFromEvents
                        ? String(sessionFromEvents.node_id)
                        : '—')}
                  </dd>
                </div>
                <div>
                  <dt className="text-slate-500">Start time (local)</dt>
                  <dd className="break-all font-mono text-xs">
                    {sessionStartedObservedAt ?? '—'}
                  </dd>
                </div>
              </dl>
            </div>
          )}
        </aside>

        {/* Conversation canvas (TASK-37.5) */}
        <main className="flex min-w-0 flex-1 flex-col overflow-hidden bg-slate-950">
          <ConversationCanvas events={events} onBlockingLockedChange={setBlockingInputLocked} />
        </main>
      </div>

      {/* Input bar */}
      <footer className="shrink-0 border-t border-slate-800 bg-slate-900/90 px-3 py-3">
        {blockingInputLocked && (
          <div className="mx-auto mb-2 max-w-6xl rounded border border-amber-700/50 bg-amber-950/60 px-3 py-2 text-center text-xs text-amber-100">
            Input is locked while a blocking security or network approval prompt is pending.
          </div>
        )}
        <div className="mx-auto flex max-w-6xl gap-2">
          <input
            type="text"
            className="min-w-0 flex-1 rounded border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 placeholder:text-slate-500 focus:border-sky-500 focus:outline-none focus:ring-1 focus:ring-sky-500"
            placeholder={
              connected
                ? blockingInputLocked
                  ? 'Resolve blocking prompt to send messages…'
                  : 'Message the agent…'
                : 'Connect the agent to send messages…'
            }
            value={draft}
            disabled={!connected || blockingInputLocked}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault()
                const text = draft.trim()
                if (!text) return
                send({ type: 'message', text })
                setDraft('')
              }
            }}
          />
          <button
            type="button"
            className="rounded bg-sky-600 px-4 py-2 text-sm font-medium text-white hover:bg-sky-500 disabled:cursor-not-allowed disabled:opacity-40"
            disabled={!connected || blockingInputLocked || !draft.trim()}
            onClick={() => {
              const text = draft.trim()
              if (!text) return
              send({ type: 'message', text })
              setDraft('')
            }}
          >
            Send
          </button>
          <button
            type="button"
            className="rounded bg-slate-700 px-4 py-2 text-sm font-medium text-white hover:bg-slate-600 disabled:cursor-not-allowed disabled:opacity-40"
            disabled={!connected}
            onClick={() => send({ type: 'cancel' })}
          >
            Cancel
          </button>
        </div>
      </footer>
    </div>
  )
}
