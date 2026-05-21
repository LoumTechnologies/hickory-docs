import { useCallback, useMemo, useRef, useState, useEffect } from 'react'
import { useAgentSocket } from './useAgentSocket'
import { SessionBundleParser } from './session-parser'
import type { UiEvent, UiCommand } from './types'
import { ConversationCanvas } from './canvas/ConversationCanvas'
import { SetupModal } from './SetupModal'

export default function App() {
  const { events: socketEvents, send, connected, sessionStartedObservedAt, isServerReplay } =
    useAgentSocket()

  const [draft, setDraft] = useState('')
  const [sidebarOpen, setSidebarOpen] = useState(true)

  // Pane state: name → { title, markdown, pane_type, entry }
  const [panes, setPanes] = useState<Map<string, { title: string; markdown: string; pane_type: string; entry: string }>>(new Map())
  const [activePaneTab, setActivePaneTab] = useState<'session' | string>('session')

  // Client-side replay state (drag-and-drop).
  const [dragReplayEvents, setDragReplayEvents] = useState<
    { event: UiEvent; timestamp: string }[] | null
  >(null)
  const [dragReplayError, setDragReplayError] = useState<string | null>(null)
  const [replayPos, setReplayPos] = useState(0)
  const [isDragOver, setIsDragOver] = useState(false)
  const dropRef = useRef<HTMLDivElement>(null)

  const isClientReplay = dragReplayEvents !== null
  const isAnyReplay = isServerReplay || isClientReplay

  // In server-replay mode the scrubber controls visible slice.
  // In drag-and-drop mode it controls the same slice over dragReplayEvents.
  const allReplayEvents: { event: UiEvent; timestamp: string }[] | null = useMemo(() => {
    if (isClientReplay) return dragReplayEvents
    if (isServerReplay) return socketEvents.map((e, i) => ({ event: e, timestamp: String(i) }))
    return null
  }, [isClientReplay, isServerReplay, dragReplayEvents, socketEvents])

  // Events actually shown (sliced by scrubber in replay modes).
  const visibleEvents: UiEvent[] = useMemo(() => {
    if (allReplayEvents !== null) {
      const pos = isServerReplay && replayPos === 0 ? allReplayEvents.length : replayPos
      return allReplayEvents.slice(0, pos).map((e) => e.event)
    }
    return socketEvents
  }, [allReplayEvents, socketEvents, isServerReplay, replayPos])

  // Scrubber max is the total number of events available.
  const scrubberMax = allReplayEvents?.length ?? 0
  const scrubberValue = isServerReplay && replayPos === 0 ? scrubberMax : replayPos

  // Update panes from pane_content events in visibleEvents.
  useEffect(() => {
    const next = new Map<string, { title: string; markdown: string; pane_type: string; entry: string }>()
    for (const ev of visibleEvents) {
      if (ev.type === 'pane_content') {
        if (ev.title === '' && ev.markdown === '') {
          // clear event — omit from map
        } else {
          next.set(ev.name, { title: ev.title, markdown: ev.markdown, pane_type: ev.pane_type, entry: ev.entry })
        }
      }
    }
    setPanes(next)
    // If the active pane was removed, reset to session tab
    setActivePaneTab((prev) => (prev !== 'session' && !next.has(prev) ? 'session' : prev))
  }, [visibleEvents])

  const relaySessionId = useMemo(() => {
    const m = window.location.pathname.match(/^\/session\/([^/]+)\/?$/)
    return m ? m[1] : null
  }, [])

  const sessionFromEvents = useMemo(() => {
    let latest: Extract<UiEvent, { type: 'session_started' }> | null = null
    for (const ev of visibleEvents) {
      if (ev.type === 'session_started') latest = ev
    }
    return latest
  }, [visibleEvents])

  const hasBlockingPrompt = useMemo(() => {
    return visibleEvents.some(
      (ev) => ev.type === 'security_violation' || ev.type === 'network_approval',
    )
  }, [visibleEvents])

  // Show the first-run setup modal when the agent is awaiting API keys.
  const awaitingApiKey = useMemo(() => {
    return visibleEvents.some((ev) => ev.type === 'awaiting_api_key')
  }, [visibleEvents])

  // ---------------------------------------------------------------------------
  // Drag-and-drop handler
  // ---------------------------------------------------------------------------

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragOver(false)
    const file = e.dataTransfer.files[0]
    if (!file) return
    const reader = new FileReader()
    reader.onload = (ev) => {
      try {
        const text = ev.target?.result
        if (typeof text !== 'string') return
        const parsed = SessionBundleParser.parse(text)
        setDragReplayEvents(parsed)
        setDragReplayError(null)
        setReplayPos(parsed.length) // start at end (show all)
      } catch (err) {
        setDragReplayError(String(err))
      }
    }
    reader.readAsText(file)
  }, [])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragOver(true)
  }, [])

  const handleDragLeave = useCallback(() => {
    setIsDragOver(false)
  }, [])

  const clearDragReplay = useCallback(() => {
    setDragReplayEvents(null)
    setDragReplayError(null)
    setReplayPos(0)
  }, [])

  // ---------------------------------------------------------------------------
  // Render
  // ---------------------------------------------------------------------------

  return (
    <div
      ref={dropRef}
      className="flex h-screen min-h-0 flex-col bg-slate-950 text-slate-100"
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
    >
      {/* First-run API key setup modal */}
      {awaitingApiKey && (
        <SetupModal
          onSubmit={(keys: { anthropic?: string; openai?: string; xai?: string }) => {
            send({ type: 'provide_api_keys', ...keys } as UiCommand)
          }}
        />
      )}

      {/* Drag-over overlay */}
      {isDragOver && !isAnyReplay && (
        <div className="pointer-events-none absolute inset-0 z-50 flex items-center justify-center bg-slate-900/80">
          <p className="rounded border border-amber-400 bg-slate-950 px-6 py-4 text-lg font-semibold text-amber-300">
            Drop session.xml to replay
          </p>
        </div>
      )}

      {/* Replay banner */}
      {isAnyReplay && (
        <div className="flex items-center gap-3 border-b border-amber-500/40 bg-amber-950/60 px-4 py-2 text-sm text-amber-100">
          <span className="font-semibold">[REPLAY]</span>
          {isClientReplay && (
            <span className="text-amber-200 text-xs">client-side (drag-and-drop)</span>
          )}
          {isServerReplay && (
            <span className="text-amber-200 text-xs">server-streamed</span>
          )}
          {isClientReplay && (
            <button
              type="button"
              onClick={clearDragReplay}
              className="ml-auto rounded border border-amber-600 bg-amber-900/50 px-2 py-0.5 text-xs text-amber-200 hover:bg-amber-800"
            >
              Clear
            </button>
          )}
        </div>
      )}

      {/* Drag-and-drop error */}
      {dragReplayError && (
        <div className="border-b border-red-500/40 bg-red-950/60 px-4 py-2 text-sm text-red-200">
          {dragReplayError}
        </div>
      )}

      {!connected && !isAnyReplay && (
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
            <div className="flex min-h-0 flex-1 flex-col space-y-3 text-sm">
              {/* Tab bar */}
              <div className="flex flex-wrap gap-1">
                <button
                  type="button"
                  className={`rounded px-2 py-0.5 text-xs ${activePaneTab === 'session' ? 'bg-sky-700 text-white' : 'bg-slate-800 text-slate-300 hover:bg-slate-700'}`}
                  onClick={() => setActivePaneTab('session')}
                >
                  Session
                </button>
                {Array.from(panes.entries()).map(([name, pane]) => (
                  <button
                    key={name}
                    type="button"
                    className={`rounded px-2 py-0.5 text-xs ${activePaneTab === name ? 'bg-sky-700 text-white' : 'bg-slate-800 text-slate-300 hover:bg-slate-700'}`}
                    onClick={() => setActivePaneTab(name)}
                  >
                    {pane.title || name}
                  </button>
                ))}
              </div>

              {activePaneTab === 'session' ? (
                <>
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
                  {!isAnyReplay && (
                    <div className="mt-4 rounded border border-dashed border-slate-600 p-3 text-center text-xs text-slate-500">
                      Drop a <code>session.xml</code> anywhere to replay
                    </div>
                  )}
                </>
              ) : (() => {
                const pane = panes.get(activePaneTab)
                if (!pane) return null
                if (pane.pane_type === 'html') {
                  return (
                    <iframe
                      src={`/pane-files/${activePaneTab}/${pane.entry}`}
                      sandbox="allow-scripts allow-forms"
                      className="min-h-0 flex-1 rounded border border-slate-700 bg-white"
                      title={pane.title || activePaneTab}
                    />
                  )
                }
                return (
                  <div className="min-h-0 flex-1 overflow-y-auto">
                    <pre className="whitespace-pre-wrap break-words text-xs text-slate-200">{pane.markdown}</pre>
                  </div>
                )
              })()}
            </div>
          )}
        </aside>

        {/* Main — conversation tree canvas */}
        <main className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden bg-slate-950">
          {visibleEvents.length === 0 ? (
            <div className="flex flex-1 items-center justify-center text-sm text-slate-500">
              {isAnyReplay
                ? 'No events in replay.'
                : connected
                  ? 'Waiting for events…'
                  : 'No events yet.'}
            </div>
          ) : (
            <ConversationCanvas
              events={visibleEvents}
              connected={connected || isAnyReplay}
              send={send}
            />
          )}
        </main>
      </div>

      {/* Playback scrubber (replay modes only) */}
      {isAnyReplay && scrubberMax > 0 && (
        <div className="shrink-0 border-t border-amber-800/40 bg-amber-950/40 px-4 py-2">
          <div className="mx-auto flex max-w-6xl items-center gap-3">
            <span className="text-xs text-amber-300 tabular-nums">
              {scrubberValue} / {scrubberMax}
            </span>
            <input
              type="range"
              min={0}
              max={scrubberMax}
              value={scrubberValue}
              onChange={(e) => setReplayPos(Number(e.target.value))}
              className="flex-1 accent-amber-400"
              aria-label="Replay position"
            />
            <button
              type="button"
              className="rounded bg-amber-800/60 px-2 py-0.5 text-xs text-amber-200 hover:bg-amber-700"
              onClick={() => setReplayPos(0)}
              title="Go to start"
            >
              ⏮
            </button>
            <button
              type="button"
              className="rounded bg-amber-800/60 px-2 py-0.5 text-xs text-amber-200 hover:bg-amber-700"
              onClick={() => setReplayPos(scrubberMax)}
              title="Go to end"
            >
              ⏭
            </button>
            {/* Show timestamp of the event at current position */}
            {isClientReplay && allReplayEvents && scrubberValue > 0 && (
              <span className="text-xs text-amber-400 tabular-nums">
                {allReplayEvents[scrubberValue - 1]?.timestamp ?? ''}
              </span>
            )}
          </div>
        </div>
      )}

      {/* Input bar — hidden in replay mode */}
      {!isAnyReplay && (
        <footer className="shrink-0 border-t border-slate-800 bg-slate-900/90 px-3 py-3">
          <div className="mx-auto flex max-w-6xl gap-2">
            <input
              type="text"
              className="min-w-0 flex-1 rounded border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-slate-100 placeholder:text-slate-500 focus:border-sky-500 focus:outline-none focus:ring-1 focus:ring-sky-500"
              placeholder={
                hasBlockingPrompt
                  ? 'Waiting for approval…'
                  : connected
                    ? 'Message the agent…'
                    : 'Connect the agent to send messages…'
              }
              value={draft}
              disabled={!connected || hasBlockingPrompt}
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
              disabled={!connected || !draft.trim() || hasBlockingPrompt}
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
      )}
    </div>
  )
}
