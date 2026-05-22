import { useState, useCallback, useRef } from 'react'

import type { UiCommand } from '../types'
import { useCollapseState } from './useCollapseState'
import { BranchColumn, type BranchColumnHandle } from './BranchColumn'
import { hasBlockingPrompts } from './HudIndicator'
import { BlockingBanner } from './BlockingBanner'
import type { BranchId, TreeModel } from './treeModel'

interface ConversationCanvasProps {
  model: TreeModel
  activeBranchId: BranchId
  setActiveBranchId: (id: BranchId) => void
  connected: boolean
  send: (cmd: UiCommand) => void
}

export function ConversationCanvas({
  model,
  activeBranchId,
  setActiveBranchId,
  connected,
  send,
}: ConversationCanvasProps) {
  const collapseState = useCollapseState(model)
  const columnRef = useRef<BranchColumnHandle>(null)
  const [activeColumnScrolledUp, setActiveColumnScrolledUp] = useState(false)

  const blocking = hasBlockingPrompts(model)

  const handleJumpToNow = useCallback(() => {
    if (activeBranchId !== model.latestBranchId) {
      setActiveBranchId(model.latestBranchId)
      setTimeout(() => columnRef.current?.scrollToBottom(), 100)
    } else {
      columnRef.current?.scrollToBottom()
    }
  }, [activeBranchId, model.latestBranchId, setActiveBranchId])

  const activeBranch = model.branches.find((b) => b.id === activeBranchId) ?? model.branches[0]
  const activeItems = model.itemsByBranch.get(activeBranchId) ?? []

  if (!activeBranch) return null

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

      <BranchColumn
        ref={columnRef}
        branch={activeBranch}
        items={activeItems}
        isActive={true}
        connected={connected}
        collapseState={collapseState}
        send={send}
        navigateToBranch={setActiveBranchId}
        onScrolledUp={setActiveColumnScrolledUp}
      />
    </div>
  )
}
