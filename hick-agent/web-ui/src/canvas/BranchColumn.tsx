import { useRef } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import type { NodeProps } from '@xyflow/react'
import type { BranchId, BranchMeta, DisplayItem } from './treeModel'
import type { CollapseState } from './useCollapseState'
import type { UiCommand } from '../types'
import { UserTurnNode } from '../nodes/UserTurnNode'
import { AssistantNode } from '../nodes/AssistantNode'
import { ThinkingNode } from '../nodes/ThinkingNode'
import { ActionPairNode } from '../nodes/ActionPairNode'
import { McpPairNode } from '../nodes/McpPairNode'
import { SpineLabelNode } from '../nodes/SpineLabelNode'
import { SubAgentNode } from '../nodes/SubAgentNode'

export const COLUMN_WIDTH = 560

export interface BranchColumnData {
  branch: BranchMeta
  items: DisplayItem[]
  isActive: boolean
  collapseState: CollapseState
  send: (cmd: UiCommand) => void
  navigateToBranch: (id: BranchId) => void
}

function NetworkApprovalItem({
  nodeId,
  host,
  suggestions,
  send,
}: {
  nodeId: number
  host: string
  suggestions: string[]
  send: (cmd: UiCommand) => void
}) {
  return (
    <div className="mx-4 my-2 rounded border border-amber-500/40 bg-amber-950/40 p-3 text-sm">
      <div className="mb-1 flex items-center gap-2 text-amber-200">
        <span>🌐</span>
        <span className="font-medium">Network approval required</span>
      </div>
      <div className="mb-2 font-mono text-xs text-amber-300">{host}</div>
      {suggestions.length > 0 && (
        <div className="mb-2 text-xs text-slate-400">
          Suggested: {suggestions.join(', ')}
        </div>
      )}
      <div className="flex gap-2">
        <button
          type="button"
          className="rounded bg-green-700 px-3 py-1 text-xs text-white hover:bg-green-600"
          onClick={() => send({ type: 'expand_node', node_id: nodeId })}
        >
          Allow
        </button>
        <button
          type="button"
          className="rounded bg-red-800 px-3 py-1 text-xs text-white hover:bg-red-700"
          onClick={() => send({ type: 'collapse_node', node_id: nodeId })}
        >
          Deny
        </button>
      </div>
    </div>
  )
}

function SecurityViolationItem({ description }: { description: string }) {
  return (
    <div className="mx-4 my-2 rounded border border-red-500/40 bg-red-950/40 p-3 text-sm">
      <div className="mb-1 flex items-center gap-2 text-red-300">
        <span>⚠</span>
        <span className="font-medium">Security violation</span>
      </div>
      <div className="text-xs text-red-200">{description}</div>
    </div>
  )
}

function AdaptiveCardItem({ json }: { json: string }) {
  return (
    <div className="mx-4 my-2 rounded border border-slate-600 bg-slate-800/50 p-3 text-xs text-slate-300">
      <div className="mb-1 text-slate-500">Adaptive Card</div>
      <pre className="max-h-32 overflow-auto font-mono text-xs">{json}</pre>
    </div>
  )
}

function renderItem(
  item: DisplayItem,
  collapseState: CollapseState,
  send: (cmd: UiCommand) => void,
  navigateToBranch: (id: BranchId) => void,
): React.ReactNode {
  switch (item.kind) {
    case 'user_turn':
      return <UserTurnNode text={item.text} />

    case 'thinking':
      return (
        <ThinkingNode
          nodeId={item.nodeId}
          isCollapsed={collapseState.isCollapsed(item.nodeId)}
          onToggle={() => collapseState.toggle(item.nodeId)}
        />
      )

    case 'assistant':
      return <AssistantNode text={item.text} streaming={item.streaming} />

    case 'action_pair':
      return (
        <ActionPairNode
          startId={item.startId}
          lang={item.lang}
          code={item.code}
          stdout={item.stdout}
          stderr={item.stderr}
          exitCode={item.exitCode}
          isCollapsed={collapseState.isCollapsed(item.startId)}
          onToggle={() => collapseState.toggle(item.startId)}
        />
      )

    case 'mcp_pair':
      return (
        <McpPairNode
          callId={item.callId}
          server={item.server}
          method={item.method}
          params={item.params}
          result={item.result}
          isCollapsed={collapseState.isCollapsed(item.callId)}
          onToggle={() => collapseState.toggle(item.callId)}
        />
      )

    case 'info':
      return <SpineLabelNode kind="info" text={item.text} />

    case 'error':
      return <SpineLabelNode kind="error" text={item.text} />

    case 'security_violation':
      return <SecurityViolationItem description={item.description} />

    case 'network_approval':
      return (
        <NetworkApprovalItem
          nodeId={item.nodeId}
          host={item.host}
          suggestions={item.suggestions}
          send={send}
        />
      )

    case 'adaptive_card':
      return <AdaptiveCardItem json={item.json} />

    case 'sub_agent_spawned':
      return (
        <SubAgentNode
          label={item.label}
          targetBranchId={item.targetBranchId}
          navigateToBranch={navigateToBranch}
        />
      )

    case 'done':
      return (
        <div className="mx-auto my-2 w-fit rounded-full border border-green-600/40 bg-green-950/40 px-4 py-1 text-xs text-green-400">
          ✓ Done
        </div>
      )

    case 'interrupted':
      return (
        <div className="mx-auto my-2 w-fit rounded-full border border-slate-600 bg-slate-800/50 px-4 py-1 text-xs text-slate-400">
          ⏹ Interrupted
        </div>
      )
  }
}

export function BranchColumn({ data }: NodeProps) {
  const { branch, items, isActive, collapseState, send, navigateToBranch } =
    data as unknown as BranchColumnData

  const scrollRef = useRef<HTMLDivElement>(null)

  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 80,
    overscan: 5,
  })

  return (
    <div
      style={{ width: COLUMN_WIDTH, height: '100vh' }}
      className={`relative flex flex-col bg-slate-950/95 transition-opacity duration-300 ${
        isActive ? 'opacity-100' : 'opacity-40'
      }`}
    >
      {/* Branch header */}
      <div className="shrink-0 border-b border-slate-800 bg-slate-900/80 px-4 py-2 text-xs font-semibold text-slate-400">
        {branch.label}
      </div>

      {/* Vertical spine line */}
      <div className="pointer-events-none absolute bottom-0 left-[25%] top-10 w-px bg-slate-800" />

      {/* Scrollable virtual list */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto overflow-x-hidden">
        <div
          style={{ height: virtualizer.getTotalSize(), position: 'relative' }}
        >
          {virtualizer.getVirtualItems().map((virtualItem) => {
            const item = items[virtualItem.index]
            return (
              <div
                key={virtualItem.key}
                data-index={virtualItem.index}
                ref={virtualizer.measureElement}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  transform: `translateY(${virtualItem.start}px)`,
                }}
              >
                {renderItem(item, collapseState, send, navigateToBranch)}
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}
