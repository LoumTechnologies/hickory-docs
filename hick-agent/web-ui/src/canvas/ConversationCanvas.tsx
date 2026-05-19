import { useState, useCallback, useRef, useMemo, useEffect } from 'react'

import type { UiEvent, UiCommand } from '../types'
import { useConversationTree } from './useConversationTree'
import { useCollapseState } from './useCollapseState'
import { usePreziController, COLUMN_WIDTH, COLUMN_GAP } from './PreziController'
import { BranchColumn, type BranchColumnHandle } from './BranchColumn'
import { BranchIndicator } from './BranchIndicator'
import { HudIndicator, hasBlockingPrompts } from './HudIndicator'
import { BlockingBanner } from './BlockingBanner'
import { Minimap } from './Minimap'
import type { BranchId } from './treeModel'

// ---------------------------------------------------------------------------
// Crosscut view
// ---------------------------------------------------------------------------

interface CrosscutViewProps {
  branchIds: BranchId[]
  onClose: (id: BranchId) => void
  model: ReturnType<typeof useConversationTree>
  collapseState: ReturnType<typeof useCollapseState>
  connected: boolean
  send: (cmd: UiCommand) => void
  navigateToBranch: (id: BranchId) => void
}

function CrosscutView({
  branchIds,
  onClose,
  model,
  collapseState,
  connected,
  send,
  navigateToBranch,
}: CrosscutViewProps) {
  return (
    <div className="flex min-h-0 flex-1 overflow-x-auto">
      {branchIds.map((bid, idx) => {
        const branch = model.branches.find((b) => b.id === bid)
        const items = model.itemsByBranch.get(bid) ?? []
        if (!branch) return null
        return (
          <div key={bid} className="flex min-h-0 flex-1 flex-row" style={{ minWidth: COLUMN_WIDTH }}>
            {idx > 0 && (
              <div className="flex shrink-0 items-center px-2 text-slate-500 select-none">
                <span className="text-lg">⚡</span>
              </div>
            )}
            <div className="flex min-h-0 flex-1 flex-col">
              <div className="flex shrink-0 items-center justify-between border-b border-slate-700 bg-slate-900 px-3 py-2 text-xs text-slate-300">
                <span className="font-semibold">{branch.label}</span>
                {branchIds.length > 1 && (
                  <button
                    type="button"
                    className="text-slate-500 hover:text-slate-200"
                    onClick={() => onClose(bid)}
                  >
                    ×
                  </button>
                )}
              </div>
              <div className="min-h-0 flex-1" style={{ height: 0 }}>
                <BranchColumn
                  branch={branch}
                  items={items}
                  isActive={true}
                  connected={connected}
                  collapseState={collapseState}
                  send={send}
                  navigateToBranch={navigateToBranch}
                />
              </div>
            </div>
          </div>
        )
      })}
    </div>
  )
}

// ---------------------------------------------------------------------------
// Main canvas
// ---------------------------------------------------------------------------

interface ConversationCanvasProps {
  events: UiEvent[]
  connected: boolean
  send: (cmd: UiCommand) => void
}

export function ConversationCanvas({ events, connected, send }: ConversationCanvasProps) {
  const model = useConversationTree(events)
  const collapseState = useCollapseState(model)
  const [activeBranchId, setActiveBranchId] = useState<BranchId>(0)
  const [crosscutBranches, setCrosscutBranches] = useState<BranchId[]>([])
  const [activeColumnScrolledUp, setActiveColumnScrolledUp] = useState(false)

  // Refs keyed by branch id for imperative scroll control
  const columnRefs = useRef<Map<BranchId, BranchColumnHandle>>(new Map())

  const { navigateToBranch: preziNavigate, tiltStyle, rowStyle } = usePreziController(
    activeBranchId,
    model.branches,
  )

  const navigateToBranch = useCallback(
    (id: BranchId) => {
      setActiveBranchId(id)
      setActiveColumnScrolledUp(false)
      preziNavigate(id)
    },
    [preziNavigate],
  )

  // Auto-navigate to new branch columns created by branch_started events
  const prevLatestBranchIdRef = useRef<BranchId>(model.latestBranchId)
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

  const handleJumpToNow = useCallback(() => {
    const latestId = model.latestBranchId
    if (latestId !== activeBranchId) {
      navigateToBranch(latestId)
      // After Prezi transition (~600ms) scroll to bottom
      setTimeout(() => {
        columnRefs.current.get(latestId)?.scrollToBottom()
      }, 650)
    } else {
      columnRefs.current.get(activeBranchId)?.scrollToBottom()
    }
  }, [model.latestBranchId, activeBranchId, navigateToBranch])

  const itemCountByBranch = useMemo(() => {
    const m = new Map<BranchId, number>()
    for (const [bid, items] of model.itemsByBranch) {
      m.set(bid, items.length)
    }
    return m
  }, [model.itemsByBranch])

  const handleCtrlClick = useCallback(
    (id: BranchId) => {
      setCrosscutBranches((prev) => {
        const next = prev.includes(id) ? prev : [...prev, id]
        if (next.length === 1 && !next.includes(activeBranchId)) {
          return [activeBranchId, id]
        }
        return next
      })
    },
    [activeBranchId],
  )

  const closeCrosscutPanel = useCallback((id: BranchId) => {
    setCrosscutBranches((prev) => prev.filter((bid) => bid !== id))
  }, [])

  const blocking = hasBlockingPrompts(model)

  // Branches sorted by column index so the flex row is in correct L→R order
  const sortedBranches = [...model.branches].sort((a, b) => a.columnIndex - b.columnIndex)

  // ── Crosscut mode ──────────────────────────────────────────────────
  if (crosscutBranches.length > 0) {
    return (
      <div className="flex min-h-0 flex-1 flex-col">
        <BlockingBanner visible={blocking} />
        <CrosscutView
          branchIds={crosscutBranches}
          onClose={closeCrosscutPanel}
          model={model}
          collapseState={collapseState}
          connected={connected}
          send={send}
          navigateToBranch={navigateToBranch}
        />
        <div className="shrink-0 border-t border-slate-800 px-3 py-1.5 text-xs text-slate-500">
          <button
            type="button"
            className="text-sky-400 hover:text-sky-300"
            onClick={() => setCrosscutBranches([])}
          >
            Exit crosscut view
          </button>
        </div>
      </div>
    )
  }

  // ── Normal single-branch view ──────────────────────────────────────
  return (
    <div className="relative flex min-h-0 flex-1 flex-col overflow-hidden">
      <BlockingBanner visible={blocking} />

      {/* Toolbar */}
      <div className="absolute left-1/2 top-2 z-30 flex -translate-x-1/2 gap-2">
        <button
          type="button"
          className="rounded border border-slate-700 bg-slate-800/90 px-3 py-1 text-xs text-slate-300 backdrop-blur hover:bg-slate-700/90"
          onClick={collapseState.collapseAll}
        >
          Collapse All
        </button>
        <button
          type="button"
          className="rounded border border-slate-700 bg-slate-800/90 px-3 py-1 text-xs text-slate-300 backdrop-blur hover:bg-slate-700/90"
          onClick={collapseState.expandAll}
        >
          Expand All
        </button>
        {activeColumnScrolledUp && (
          <button
            type="button"
            className="animate-pulse rounded border border-sky-600 bg-sky-900/90 px-3 py-1 text-xs text-sky-200 backdrop-blur hover:bg-sky-800/90"
            onClick={handleJumpToNow}
          >
            ↓ Jump to now
          </button>
        )}
      </div>

      {/* 3D-tilt wrapper (Prezi zoom-out effect) */}
      <div style={{ ...tiltStyle, flex: '1', minHeight: 0, overflow: 'hidden' }}>
        {/* Sliding column row */}
        <div
          style={{
            ...rowStyle,
            display: 'flex',
            flexDirection: 'row',
            alignItems: 'stretch',
            height: '100%',
            gap: COLUMN_GAP,
          }}
        >
          {sortedBranches.map((branch) => (
            <BranchColumn
              key={branch.id}
              ref={(handle) => {
                if (handle) columnRefs.current.set(branch.id, handle)
                else columnRefs.current.delete(branch.id)
              }}
              branch={branch}
              items={model.itemsByBranch.get(branch.id) ?? []}
              isActive={branch.id === activeBranchId}
              connected={connected}
              collapseState={collapseState}
              send={send}
              navigateToBranch={navigateToBranch}
              onScrolledUp={branch.id === activeBranchId ? setActiveColumnScrolledUp : undefined}
            />
          ))}
        </div>
      </div>

      <Minimap
        branches={model.branches}
        activeBranchId={activeBranchId}
        itemCountByBranch={itemCountByBranch}
        navigateToBranch={navigateToBranch}
      />
      <BranchIndicator
        activeBranchId={activeBranchId}
        branches={model.branches}
        navigateToBranch={navigateToBranch}
        onCtrlClick={handleCtrlClick}
      />
      <HudIndicator
        model={model}
        activeBranchId={activeBranchId}
        navigateToBranch={navigateToBranch}
      />
    </div>
  )
}
