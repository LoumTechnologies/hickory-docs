import { useCallback, useMemo, useRef, useState, useEffect } from 'react'
import { useAgentSocket } from './useAgentSocket'
import { SessionBundleParser } from './session-parser'
import type { UiEvent, UiCommand } from './types'
import { useConversationTree } from './canvas/useConversationTree'
import { ConversationCanvas } from './canvas/ConversationCanvas'
import { SetupModal } from './SetupModal'
import type { BranchId } from './canvas/treeModel'

export default function App() {
  const { events: socketEvents, send, connected, sessionStartedObservedAt, isServerReplay } =
    useAgentSocket()

  const [draft, setDraft] = useState('')
  const [sidebarOpen, setSidebarOpen] = useState(true)
  const [activeBranchId, setActiveBranchId] = useState<BranchId>(0)
  const navigateToBranch = useCallback((id: BranchId) => setActiveBranchId(id), [])

  // Pane state: name → { title, markdown, pane_type, entry }
  const [panes, setPanes] = useState<Map<string, { title: string; markdown: string; pane_type: string; entry: string }>>(new Map())
  const [activePaneTab, setActivePaneTab] = useState<'conversations' | string>('conversations')

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

  const allReplayEvents: { event: UiEvent; timestamp: string }[] | null = useMemo(() => {
    if (isClientReplay) return dragReplayEvents
    if (isServerReplay) return socketEvents.map((e, i) => ({ event: e, timestamp: String(i) }))
    return null
  }, [isClientReplay, isServerReplay, dragReplayEvents, socketEvents])

  const visibleEvents: UiEvent[] = useMemo(() => {
    if (allReplayEvents !== null) {
      const pos = isServerReplay && replayPos === 0 ? allReplayEvents.length : replayPos
      return allReplayEvents.slice(0, pos).map((e) => e.event)
    }
    return socketEvents
  }, [allReplayEvents, socketEvents, isServerReplay, replayPos])

  const scrubberMax = allReplayEvents?.length ?? 0
  const scrubberValue = isServerReplay && replayPos === 0 ? scrubberMax : replayPos

  // Build conversation model from events.
  const model = useConversationTree(visibleEvents)

  // Auto-navigate to newly created rewind branches.
  const prevLatestBranchIdRef = useRef<BranchId>(0)
  useEffect(() => {
    const latestId = model.latestBranchId
    if (latestId !== prevLatestBranchIdRef.current) {
      prevLatestBranchIdRef.current = latestId
      const branch = model.branches.find((b) => b.id === latestId)
      if (branch?.kind === 'branch') {
        navigateToBranch(latestId)
      }
    }
  })

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
    setActivePaneTab((prev) => (prev !== 'conversations' && !next.has(prev) ? 'conversations' : prev))
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
        setReplayPos(parsed.length)
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

  const sendMessage = useCallback(() => {
    const text = draft.trim()
    if (!text) return
    send({ type: 'message', text })
    setDraft('')
  }, [draft, send])

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
      {awaitingApiKey && (
        <SetupModal
          onSubmit={(keys: { anthropic?: string; openai?: string; xai?: string }) => {
            send({ type: 'provide_api_keys', ...keys } as UiCommand)
          }}
        />
      )}

      {isDragOver && !isAnyReplay && (
        <div className="pointer-events-none absolute inset-0 z-50 flex items-center justify-center bg-slate-900/80">
          <p className="rounded border border-amber-400 bg-slate-950 px-6 py-4 text-lg font-semibold text-amber-300">
            Drop session.xml to replay
          </p>
        </div>
      )}

      {isAnyReplay && (
        <div className="flex items-center gap-3 border-b border-amber-500/40 bg-amber-950/60 px-4 py-2 text-sm text-amber-100">
          <span className="font-semibold">[REPLAY]</span>
          {isClientReplay && <span className="text-xs text-amber-200">client-side (drag-and-drop)</span>}
          {isServerReplay && <span className="text-xs text-amber-200">server-streamed</span>}
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
            sidebarOpen ? 'w-64 px-3 py-3' : 'w-11 overflow-hidden px-1 py-2'
          }`}
        >
          <button
            type="button"
            className="mb-3 rounded border border-slate-700 bg-slate-800 px-2 py-1 text-left text-xs text-slate-300 hover:bg-slate-700"
            onClick={() => setSidebarOpen((o) => !o)}
            title={sidebarOpen ? 'Collapse sidebar' : 'Expand sidebar'}
          >
            {sidebarOpen ? '◀' : '▶'}
          </button>

          {sidebarOpen && (
            <div className="flex min-h-0 flex-1 flex-col gap-2 text-sm">
              {/* Tab bar — Conversations + any agent panes */}
              {panes.size > 0 && (
                <div className="flex flex-wrap gap-1">
                  <button
                    type="button"
                    className={`rounded px-2 py-0.5 text-xs ${activePaneTab === 'conversations' ? 'bg-sky-700 text-white' : 'bg-slate-800 text-slate-300 hover:bg-slate-700'}`}
                    onClick={() => setActivePaneTab('conversations')}
                  >
                    Conversations
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
              )}

              {activePaneTab === 'conversations' ? (
                <>
                  {relaySessionId && (
                    <p className="rounded bg-indigo-950/50 px-2 py-1 text-xs text-indigo-200">
                      Remote relay session
                    </p>
                  )}

                  {/* Conversation (branch) list */}
                  <div className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto">
                    <p className="mb-1 shrink-0 text-[10px] font-semibold uppercase tracking-wider text-slate-500">
                      Conversations
                    </p>
                    {model.branches.map((branch) => {
                      const itemCount = model.itemsByBranch.get(branch.id)?.length ?? 0
                      const isActive = branch.id === activeBranchId
                      const kindIcon =
                        branch.kind === 'sub_agent' ? '🤖' : branch.kind === 'branch' ? '⎇' : '💬'
                      return (
                        <button
                          key={branch.id}
                          type="button"
                          onClick={() => navigateToBranch(branch.id)}
                          className={`flex w-full items-center gap-2 rounded px-2 py-2 text-left text-xs transition-colors ${
                            isActive
                              ? 'bg-sky-800/60 text-sky-100'
                              : 'text-slate-300 hover:bg-slate-800 hover:text-slate-100'
                          }`}
                        >
                          <span className="shrink-0">{kindIcon}</span>
                          <span className="min-w-0 flex-1 truncate">{branch.label}</span>
                          <span className="shrink-0 tabular-nums text-[10px] text-slate-500">
                            {itemCount}
                          </span>
                        </button>
                      )
                    })}
                  </div>

                  {/* Session metadata */}
                  <div className="shrink-0 border-t border-slate-800 pt-2">
                    <dl className="space-y-1.5 text-slate-400">
                      <div>
                        <dt className="text-[10px] uppercase tracking-wider text-slate-600">Model</dt>
                        <dd className="truncate font-mono text-[11px]">{sessionFromEvents?.model ?? '—'}</dd>
                      </div>
                      <div>
                        <dt className="text-[10px] uppercase tracking-wider text-slate-600">Provider</dt>
                        <dd className="truncate font-mono text-[11px]">{sessionFromEvents?.provider_info ?? '—'}</dd>
                      </div>
                      <div>
                        <dt className="text-[10px] uppercase tracking-wider text-slate-600">Session</dt>
                        <dd className="truncate font-mono text-[11px]">
                          {relaySessionId ?? (sessionFromEvents ? String(sessionFromEvents.node_id) : '—')}
                        </dd>
                      </div>
                      {sessionStartedObservedAt && (
                        <div>
                          <dt className="text-[10px] uppercase tracking-wider text-slate-600">Started</dt>
                          <dd className="truncate font-mono text-[11px]">{sessionStartedObservedAt}</dd>
                        </div>
                      )}
                    </dl>
                    {!isAnyReplay && (
                      <div className="mt-3 rounded border border-dashed border-slate-700 p-2 text-center text-[11px] text-slate-600">
                        Drop a <code>session.xml</code> to replay
                      </div>
                    )}
                  </div>
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

        {/* Main — conversation view */}
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
              model={model}
              activeBranchId={activeBranchId}
              setActiveBranchId={navigateToBranch}
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
            <span className="tabular-nums text-xs text-amber-300">
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
            {isClientReplay && allReplayEvents && scrubberValue > 0 && (
              <span className="tabular-nums text-xs text-amber-400">
                {allReplayEvents[scrubberValue - 1]?.timestamp ?? ''}
              </span>
            )}
          </div>
        </div>
      )}

      {/* Input bar — hidden in replay mode */}
      {!isAnyReplay && (
        <footer className="shrink-0 border-t border-slate-800 bg-slate-900/90 px-4 py-3">
          <div className="mx-auto flex max-w-4xl items-end gap-2">
            <textarea
              rows={1}
              className="block min-w-0 flex-1 resize-none rounded-2xl border border-sky-500/40 bg-sky-900/60 px-4 py-3 text-sm text-sky-100 placeholder:text-slate-500 focus:border-sky-400 focus:outline-none focus:ring-1 focus:ring-sky-400 disabled:cursor-not-allowed disabled:opacity-40"
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
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  sendMessage()
                }
              }}
            />
            <div className="flex shrink-0 flex-col gap-1">
              <button
                type="button"
                className="rounded-xl bg-sky-600 px-4 py-2 text-sm font-medium text-white hover:bg-sky-500 disabled:cursor-not-allowed disabled:opacity-40"
                disabled={!connected || !draft.trim() || hasBlockingPrompt}
                onClick={sendMessage}
              >
                Send
              </button>
              <button
                type="button"
                className="rounded-xl bg-slate-700 px-4 py-2 text-sm font-medium text-white hover:bg-slate-600 disabled:cursor-not-allowed disabled:opacity-40"
                disabled={!connected}
                onClick={() => send({ type: 'cancel' })}
              >
                Cancel
              </button>
            </div>
          </div>
        </footer>
      )}
    </div>
  )
}
