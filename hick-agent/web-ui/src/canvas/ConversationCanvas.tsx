import { useState, useMemo, useCallback } from 'react'
import {
  ReactFlow,
  ReactFlowProvider,
  MiniMap,
  type Node,
  type Edge,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'

import type { UiEvent, UiCommand } from '../types'
import { useConversationTree } from './useConversationTree'
import { useCollapseState } from './useCollapseState'
import { usePreziController, COLUMN_WIDTH, COLUMN_GAP } from './PreziController'
import { BranchColumn, type BranchColumnData } from './BranchColumn'
import { BranchIndicator } from './BranchIndicator'
import { HudIndicator, hasBlockingPrompts } from './HudIndicator'
import { BlockingBanner } from './BlockingBanner'
import type { BranchId } from './treeModel'

const nodeTypes = { branchColumn: BranchColumn }

interface CrosscutPanelProps {
  branchIds: BranchId[]
  onClose: (id: BranchId) => void
  model: ReturnType<typeof useConversationTree>
  collapseState: ReturnType<typeof useCollapseState>
  send: (cmd: UiCommand) => void
  navigateToBranch: (id: BranchId) => void
}

function CrosscutPanel({ branchIds, onClose, model }: CrosscutPanelProps) {
  return (
    <div className="flex h-full min-h-0 flex-1 overflow-x-auto">
      {branchIds.map((bid, idx) => {
        const branch = model.branches.find((b) => b.id === bid)
        const items = model.itemsByBranch.get(bid) ?? []
        if (!branch) return null
        return (
          <div key={bid} className="flex min-h-0 flex-col" style={{ width: COLUMN_WIDTH, flexShrink: 0 }}>
            {idx > 0 && (
              <div className="flex h-full items-center px-1 text-slate-500">
                <span className="text-lg">⚡</span>
              </div>
            )}
            <div className="flex items-center justify-between border-b border-slate-700 bg-slate-900 px-3 py-2 text-xs text-slate-300">
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
            <div className="flex-1 overflow-y-auto">
              {items.map((item, i) => {
                // Import renderItem logic inline by using BranchColumn data
                const nodeId = 'nodeId' in item ? item.nodeId : 'startId' in item ? item.startId : 'callId' in item ? item.callId : i
                return (
                  <div key={nodeId + '-' + i}>
                    {/* Simplified rendering for crosscut panels */}
                    <div className="px-2 py-1 text-xs text-slate-400">
                      [{item.kind}]
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
        )
      })}
    </div>
  )
}

interface InnerCanvasProps {
  events: UiEvent[]
  connected: boolean
  send: (cmd: UiCommand) => void
}

function InnerCanvas({ events, send }: InnerCanvasProps) {
  const model = useConversationTree(events)
  const collapseState = useCollapseState(model)
  const [activeBranchId, setActiveBranchId] = useState<BranchId>(0)
  const [crosscutBranches, setCrosscutBranches] = useState<BranchId[]>([])

  const { navigateToBranch: preziNavigate, tiltStyle } = usePreziController(
    activeBranchId,
    model.branches,
  )

  const navigateToBranch = useCallback(
    (id: BranchId) => {
      setActiveBranchId(id)
      preziNavigate(id)
    },
    [preziNavigate],
  )

  const handleCtrlClick = useCallback((id: BranchId) => {
    setCrosscutBranches((prev) => (prev.includes(id) ? prev : [...prev, id]))
    if (crosscutBranches.length === 0) {
      // Also add current active branch if entering crosscut
      setCrosscutBranches((prev) => {
        const result = [...prev]
        if (!result.includes(activeBranchId)) result.unshift(activeBranchId)
        if (!result.includes(id)) result.push(id)
        return result
      })
    }
  }, [crosscutBranches, activeBranchId])

  const closeCrosscutPanel = useCallback((id: BranchId) => {
    setCrosscutBranches((prev) => {
      const next = prev.filter((bid) => bid !== id)
      return next
    })
  }, [])

  const blocking = hasBlockingPrompts(model)

  // Build React Flow nodes
  const nodes: Node[] = useMemo(
    () =>
      model.branches.map((branch) => {
        const data: BranchColumnData = {
          branch,
          items: model.itemsByBranch.get(branch.id) ?? [],
          isActive: branch.id === activeBranchId,
          collapseState,
          send,
          navigateToBranch,
        }
        return {
          id: String(branch.id),
          type: 'branchColumn',
          position: { x: branch.columnIndex * (COLUMN_WIDTH + COLUMN_GAP), y: 0 },
          data: data as unknown as Record<string, unknown>,
          draggable: false,
          selectable: false,
        }
      }),
    [model, activeBranchId, collapseState, send, navigateToBranch],
  )

  // Build edges from parent branches to child branches
  const edges: Edge[] = useMemo(
    () =>
      model.branches
        .filter((b) => b.parentBranchId !== null)
        .map((b) => ({
          id: `edge-${b.parentBranchId}-${b.id}`,
          source: String(b.parentBranchId),
          target: String(b.id),
          type: 'smoothstep',
          style: { stroke: '#475569', strokeWidth: 1, strokeDasharray: '4 4' },
        })),
    [model.branches],
  )

  if (crosscutBranches.length > 0) {
    return (
      <div className="flex min-h-0 flex-1 flex-col">
        <BlockingBanner visible={blocking} />
        <div className="flex min-h-0 flex-1 overflow-hidden">
          <CrosscutPanel
            branchIds={crosscutBranches}
            onClose={closeCrosscutPanel}
            model={model}
            collapseState={collapseState}
            send={send}
            navigateToBranch={navigateToBranch}
          />
        </div>
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
      </div>

      {/* 3D tilt wrapper */}
      <div style={tiltStyle} className="min-h-0 flex-1">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          panOnDrag={false}
          panOnScroll={false}
          zoomOnScroll={false}
          zoomOnPinch={false}
          zoomOnDoubleClick={false}
          preventScrolling={true}
          nodesDraggable={false}
          nodesConnectable={false}
          elementsSelectable={false}
          fitView
          fitViewOptions={{ padding: 0 }}
          defaultViewport={{ x: 0, y: 0, zoom: 1 }}
          style={{ background: 'rgb(2 6 23)' }}
        >
          <MiniMap
            style={{ background: '#0f172a' }}
            maskColor="rgba(15, 23, 42, 0.7)"
            nodeColor="#334155"
          />
        </ReactFlow>
      </div>

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

interface ConversationCanvasProps {
  events: UiEvent[]
  connected: boolean
  send: (cmd: UiCommand) => void
}

export function ConversationCanvas({ events, connected, send }: ConversationCanvasProps) {
  return (
    <ReactFlowProvider>
      <InnerCanvas events={events} connected={connected} send={send} />
    </ReactFlowProvider>
  )
}
