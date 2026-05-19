import { useVirtualizer } from '@tanstack/react-virtual'
import { Handle, Position, type Node, type NodeProps } from '@xyflow/react'
import { memo, useCallback, useMemo, useRef } from 'react'
import { renderConversationRow } from './RowViews'
import type { ConversationNodeState } from './treeModel'

export type BranchColumnData = {
  branchKey: number | null
  nodeIds: readonly number[]
  nodes: ReadonlyMap<number, ConversationNodeState>
  expandedOf: (id: number) => boolean
  toggleExpanded: (id: number) => void
  /** Zoomed-out simplified placeholders */
  ambient: boolean
  turnSepLabels: ReadonlyMap<number, string | undefined>
  /** 1 active column, ~0.4 siblings */
  columnOpacity: number
}

export type BranchRfNode = Node<BranchColumnData, 'branchColumn'>

function estimateHeight(id: number, nodes: ReadonlyMap<number, ConversationNodeState>, ambient: boolean): number {
  if (ambient) return 52
  const n = nodes.get(id)
  if (!n) return 48
  switch (n.kind) {
    case 'session':
      return 96
    case 'user_turn':
    case 'assistant':
      return Math.min(420, Math.max(72, 40 + String(n.text ?? '').length / 6))
    case 'thinking':
      return 72
    case 'action':
      return n.collapsed_by_default ? 96 : 220
    case 'mcp_pair':
      return n.collapsed_by_default ? 88 : 200
    case 'security_violation':
    case 'network_approval':
      return 120
    case 'adaptive_card':
      return 160
    case 'info':
    case 'error':
      return 40
    case 'sub_agent':
      return 56
    default:
      return 48
  }
}

export const BranchColumnBody = memo(function BranchColumnBody({
  branchKey,
  nodeIds,
  nodes,
  expandedOf,
  toggleExpanded,
  ambient,
  turnSepLabels,
  columnOpacity,
}: Omit<BranchColumnData, never>) {
  const scrollRef = useRef<HTMLDivElement | null>(null)

  const measures = useMemo(
    () => nodeIds.map((id) => estimateHeight(id, nodes, ambient)),
    [ambient, nodeIds, nodes],
  )

  const virtualizer = useVirtualizer({
    count: nodeIds.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: (i) => measures[i] ?? 56,
    overscan: 6,
  })

  const onToggle = useCallback(
    (id: number) => {
      toggleExpanded(id)
    },
    [toggleExpanded],
  )

  const slug = branchKey === null ? 'main' : String(branchKey)
  const label = branchKey === null ? 'Main thread' : `Branch · ${branchKey}`

  return (
    <div
      className="rounded-xl border border-slate-800 bg-slate-950/95 shadow-xl transition-opacity duration-300"
      style={{
        width: 420,
        height: 'calc(100vh - 220px)',
        minHeight: 320,
        opacity: columnOpacity,
      }}
    >
      <div className="border-b border-slate-800 px-3 py-2 text-[11px] font-semibold uppercase tracking-wide text-slate-400">
        {label}
      </div>

      <div
        ref={scrollRef}
        data-branch-scroll={slug}
        className="h-[calc(100%-40px)] overflow-y-auto overflow-x-hidden"
      >
        <div style={{ height: virtualizer.getTotalSize(), position: 'relative' }}>
          {virtualizer.getVirtualItems().map((vi) => {
            const id = nodeIds[vi.index]
            const node = nodes.get(id)
            if (!node) return null
            return (
              <div
                key={id}
                data-row-node-id={id}
                ref={virtualizer.measureElement}
                data-index={vi.index}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  transform: `translateY(${vi.start}px)`,
                }}
              >
                {renderConversationRow(node, {
                  expanded: expandedOf(id),
                  onToggle,
                  showTurnSep: turnSepLabels.get(id),
                  placeholder: ambient,
                })}
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
})

export const BranchColumnNode = memo(function BranchColumnNode({ data }: NodeProps<BranchRfNode>) {
  return (
    <div>
      <Handle type="target" position={Position.Left} className="!size-2 !opacity-0" />
      <Handle type="source" position={Position.Right} className="!size-2 !opacity-0" />
      <BranchColumnBody
        branchKey={data.branchKey}
        nodeIds={data.nodeIds}
        nodes={data.nodes}
        expandedOf={data.expandedOf}
        toggleExpanded={data.toggleExpanded}
        ambient={data.ambient}
        turnSepLabels={data.turnSepLabels}
        columnOpacity={data.columnOpacity}
      />
    </div>
  )
})
