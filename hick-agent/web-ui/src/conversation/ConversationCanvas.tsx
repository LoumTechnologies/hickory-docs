import {
  Background,
  MiniMap,
  Panel,
  ReactFlow,
  ReactFlowProvider,
  useReactFlow,
  type NodeTypes,
  type ReactFlowInstance,
  type OnNodesChange,
  type XYPosition,
  applyNodeChanges,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { useCallback, useEffect, useMemo, useState, type MouseEvent as ReactMouseEvent } from 'react'
import type { UiEvent } from '../types'
import { BranchColumnBody, BranchColumnNode, type BranchRfNode } from './BranchColumnNode'
import {
  branchColumnOf,
  branchColumnsInOrder,
  ingestEvents,
  isBlockingKind,
  orderedBranchNodes,
  type ConversationNodeState,
} from './treeModel'
import { usePreziNavigator } from './usePreziNavigator'

const COL_W = 420
const GAP = 56

const nodeTypes = { branchColumn: BranchColumnNode } as NodeTypes

function branchSlug(k: number | null): string {
  return k === null ? 'main' : String(k)
}

function buildTurnSepLabels(
  ids: readonly number[],
  nodes: ReadonlyMap<number, ConversationNodeState>,
): Map<number, string | undefined> {
  const m = new Map<number, string | undefined>()
  let prev: ConversationNodeState | undefined
  const times = new Map<number, string>()
  for (const id of ids) {
    const n = nodes.get(id)
    if (!n) continue
    if (prev?.kind === 'user_turn' && (n.kind === 'assistant' || n.kind === 'thinking')) {
      if (!times.has(id)) times.set(id, new Date().toLocaleTimeString())
      m.set(id, `── ${times.get(id)} ──`)
    }
    prev = n
  }
  return m
}

function FlowLogic({
  events,
  tree,
  expandedOf,
  toggleExpanded,
  toggleCrosscutBranch,
  resolvedBlocking,
  setRfNodes,
  onTilt,
  cancelRev,
}: {
  events: readonly UiEvent[]
  tree: ReturnType<typeof ingestEvents>
  expandedOf: (id: number) => boolean
  toggleExpanded: (id: number) => void
  toggleCrosscutBranch: (b: number | null, additive: boolean) => void
  resolvedBlocking: ReadonlySet<number>
  setRfNodes: React.Dispatch<React.SetStateAction<BranchRfNode[]>>
  onTilt: (rx: number, ry: number) => void
  cancelRev: number
}) {
  const rf = useReactFlow() as unknown as ReactFlowInstance
  const getContainer = useCallback(() => document.querySelector('.hick-flow-surface') as HTMLElement | null, [])

  const [navigatedBranch, setNavigatedBranch] = useState<number | null>(null)
  const [ambientZoom] = useState(false)
  const [scrollAwayFromBottom, setScrollAwayFromBottom] = useState(false)
  const [jumpNote, setJumpNote] = useState(false)

  const cols = useMemo(
    () => branchColumnsInOrder(tree.nodes, tree.sessionRootId, tree.children),
    [tree.children, tree.nodes, tree.sessionRootId],
  )

  const fallbackBranch = useMemo(() => {
    if (!cols.length) return null
    return cols.includes(null) ? null : cols[0] ?? null
  }, [cols])

  const activeBranch = navigatedBranch ?? fallbackBranch

  const ai = Math.max(0, cols.indexOf(activeBranch ?? null))

  const columnXs = useMemo(() => cols.map((_, i) => (i - ai) * (COL_W + GAP)), [ai, cols])

  const branchBlocks = useMemo(() => {
    const blocks = new Map<number | null, { nodeIds: number[]; sep: Map<number, string | undefined> }>()
    for (const col of cols) {
      const nodeIds = orderedBranchNodes(tree.nodes, tree.sessionRootId, col, tree.children)
      blocks.set(col, { nodeIds, sep: buildTurnSepLabels(nodeIds, tree.nodes) })
    }
    return blocks
  }, [cols, tree.children, tree.nodes, tree.sessionRootId])

  const rfNodes: BranchRfNode[] = useMemo(
    () =>
      cols.map((col, i) => ({
        id: `col-${branchSlug(col)}`,
        type: 'branchColumn',
        position: { x: columnXs[i] ?? 0, y: 0 },
        draggable: false,
        selectable: false,
        data: {
          branchKey: col,
          nodeIds: branchBlocks.get(col)?.nodeIds ?? [],
          nodes: tree.nodes,
          expandedOf,
          toggleExpanded,
          ambient: ambientZoom,
          turnSepLabels: branchBlocks.get(col)?.sep ?? new Map(),
          columnOpacity: col === (activeBranch ?? null) ? 1 : 0.42,
        },
      })),
    [
      activeBranch,
      ambientZoom,
      branchBlocks,
      cols,
      columnXs,
      expandedOf,
      toggleExpanded,
      tree.nodes,
    ],
  )

  useEffect(() => {
    setRfNodes(rfNodes)
  }, [rfNodes, setRfNodes])

  const latestBranch = ((): number | null => {
    for (let i = events.length - 1; i >= 0; i--) {
      const ev = events[i]
      if (!('node_id' in ev)) continue
      const nid = ev.node_id as number
      if (!tree.nodes.has(nid)) continue
      return branchColumnOf(tree.nodes, nid)
    }
    return null
  })()

  const blockingNodes = useMemo(() => {
    const pending: ConversationNodeState[] = []
    for (const n of tree.nodes.values()) {
      if (isBlockingKind(n.kind) && !resolvedBlocking.has(n.id)) pending.push(n)
    }
    return pending
  }, [resolvedBlocking, tree.nodes])

  const { navigate, cancel, viewportCenterOn } = usePreziNavigator(rf, getContainer, onTilt)

  useEffect(() => {
    if (cancelRev <= 0) return
    cancel()
  }, [cancelRev, cancel])

  const flyToBranch = useCallback(
    (target: number | null, dirHint?: number) => {
      const idx = cols.indexOf(target)
      if (idx < 0) return
      const cx = (columnXs[idx] ?? 0) + COL_W / 2
      const cy = 280
      const targetVp = viewportCenterOn(cx, cy, 1)
      const curIdx = cols.indexOf(activeBranch ?? null)
      const dir = dirHint ?? (idx === curIdx ? 1 : idx > curIdx ? 1 : -1)
      setNavigatedBranch(target)
      navigate(targetVp, dir)
    },
    [activeBranch, cols, columnXs, navigate, viewportCenterOn],
  )

  const onMiniMapClick = useCallback(
    (_ev: ReactMouseEvent<Element>, pos: XYPosition) => {
      let best = 0
      let bestD = Infinity
      cols.forEach((_, i) => {
        const cx = (columnXs[i] ?? 0) + COL_W / 2
        const d = Math.abs(cx - pos.x)
        if (d < bestD) {
          bestD = d
          best = i
        }
      })
      const target = cols[best]
      if (target !== undefined) flyToBranch(target)
    },
    [cols, columnXs, flyToBranch],
  )

  const onPaneClick = useCallback(
    (ev: React.MouseEvent) => {
      if (!ev.ctrlKey && !ev.metaKey) return
      ev.preventDefault()
      const surf = document.querySelector('.hick-flow-surface')
      const rect = surf?.getBoundingClientRect()
      if (!rect) return
      const p = rf.screenToFlowPosition({ x: ev.clientX - rect.left, y: ev.clientY - rect.top })
      let best = 0
      let bestD = Infinity
      cols.forEach((_, i) => {
        const cx = (columnXs[i] ?? 0) + COL_W / 2
        const d = Math.abs(cx - p.x)
        if (d < bestD) {
          bestD = d
          best = i
        }
      })
      const target = cols[best]
      if (target !== undefined) toggleCrosscutBranch(target, true)
    },
    [cols, columnXs, rf, toggleCrosscutBranch],
  )

  const jumpToNow = useCallback(() => {
    const targetBr = latestBranch ?? activeBranch ?? null
    if (targetBr !== activeBranch) {
      setJumpNote(true)
      flyToBranch(targetBr ?? null)
      window.setTimeout(() => setJumpNote(false), 900)
    }
    const slug = branchSlug(activeBranch ?? null)
    const el = document.querySelector(`[data-branch-scroll="${slug}"]`)
    if (el) el.scrollTop = el.scrollHeight
    setScrollAwayFromBottom(false)
  }, [activeBranch, flyToBranch, latestBranch])

  useEffect(() => {
    const slug = branchSlug(activeBranch ?? null)
    const el = document.querySelector(`[data-branch-scroll="${slug}"]`)
    if (!el) return
    const onScroll = () => {
      const gap = el.scrollHeight - el.scrollTop - el.clientHeight
      setScrollAwayFromBottom(gap > 48)
    }
    el.addEventListener('scroll', onScroll, { passive: true })
    onScroll()
    return () => el.removeEventListener('scroll', onScroll)
  }, [activeBranch, events.length])

  useEffect(() => {
    if (scrollAwayFromBottom) return
    const slug = branchSlug(activeBranch ?? null)
    const el = document.querySelector(`[data-branch-scroll="${slug}"]`)
    if (el) el.scrollTop = el.scrollHeight
  }, [activeBranch, events.length, scrollAwayFromBottom])

  const leftBranches = cols.filter((_, i) => i < ai)
  const rightBranches = cols.filter((_, i) => i > ai)
  const overflowLeft = Math.max(0, leftBranches.length - 2)
  const overflowRight = Math.max(0, rightBranches.length - 2)

  const blockingEdge = useMemo(() => {
    if (blockingNodes.length === 0) return null
    const b = blockingNodes[0]
    const br = branchColumnOf(tree.nodes, b.id)
    const bi = cols.indexOf(br)
    if (bi < 0) return { side: 'right' as const, target: br }
    if (bi < ai) return { side: 'left' as const, target: br }
    if (bi > ai) return { side: 'right' as const, target: br }
    return null
  }, [ai, blockingNodes, cols, tree.nodes])

  const ctrlClickBranch = useCallback(
    (br: number | null, ev: React.MouseEvent) => {
      if (ev.ctrlKey || ev.metaKey) {
        ev.preventDefault()
        toggleCrosscutBranch(br, true)
      } else {
        flyToBranch(br, br !== null && cols.indexOf(br) < ai ? -1 : 1)
      }
    },
    [cols, ai, flyToBranch, toggleCrosscutBranch],
  )

  return (
    <>
      <Background color="#1e293b" gap={20} />
      <MiniMap
        className="!bg-slate-900/90"
        zoomable={false}
        pannable={false}
        onClick={onMiniMapClick}
        nodeStrokeWidth={2}
        maskColor="rgba(15,23,42,0.5)"
      />

      <Panel position="center-left" className="pointer-events-none m-2 flex flex-col gap-2">
        {leftBranches.slice(-2).map((br) => (
          <button
            key={branchSlug(br)}
            type="button"
            className="pointer-events-auto rounded border border-slate-600/80 bg-slate-950/85 px-2 py-3 text-[11px] text-slate-200 opacity-80 hover:bg-slate-800 hover:opacity-100"
            title="Navigate · Ctrl+click adds crosscut panel"
            onClick={(ev) => ctrlClickBranch(br, ev)}
          >
            ◀ {branchSlug(br)}
          </button>
        ))}
        {overflowLeft > 0 && (
          <span className="rounded bg-slate-900/90 px-2 py-1 text-[10px] text-slate-500">+{overflowLeft}</span>
        )}
      </Panel>

      <Panel position="center-right" className="pointer-events-none m-2 flex flex-col gap-2">
        {rightBranches.slice(0, 2).map((br) => (
          <button
            key={branchSlug(br)}
            type="button"
            className="pointer-events-auto rounded border border-slate-600/80 bg-slate-950/85 px-2 py-3 text-[11px] text-slate-200 opacity-80 hover:bg-slate-800 hover:opacity-100"
            title="Navigate · Ctrl+click adds crosscut panel"
            onClick={(ev) => ctrlClickBranch(br, ev)}
          >
            {branchSlug(br)} ▶
          </button>
        ))}
        {overflowRight > 0 && (
          <span className="rounded bg-slate-900/90 px-2 py-1 text-[10px] text-slate-500">+{overflowRight}</span>
        )}
      </Panel>

      {blockingEdge && (
        <Panel position="top-center" className="pointer-events-none mt-36">
          <button
            type="button"
            className="pointer-events-auto flex animate-pulse items-center gap-2 rounded-full border border-amber-400/60 bg-amber-950/95 px-3 py-2 text-xs text-amber-100 shadow-xl"
            onClick={() => flyToBranch(blockingEdge.target)}
          >
            <span>⚠</span>
            Blocking prompt · jump to branch column
          </button>
        </Panel>
      )}

      {scrollAwayFromBottom && (
        <Panel position="bottom-center" className="pointer-events-none mb-10">
          <button
            type="button"
            className="pointer-events-auto rounded-full border border-sky-600 bg-sky-900/95 px-4 py-2 text-xs font-medium text-sky-50 shadow-xl hover:bg-sky-800"
            onClick={jumpToNow}
          >
            Jump to now{jumpNote ? ' · switching branch' : ''}
          </button>
        </Panel>
      )}

      {/* Expose pane handler to parent ReactFlow via custom event bridge */}
      <PaneClickBridge onPaneClick={onPaneClick} />
    </>
  )
}

/** Hooks into ReactFlow's pane without duplicating ReactFlow (listens capture on document). */
function PaneClickBridge({ onPaneClick }: { onPaneClick: (ev: React.MouseEvent) => void }) {
  useEffect(() => {
    const handler = (ev: Event) => {
      if (!(ev instanceof MouseEvent)) return
      const t = ev.target
      if (!(t instanceof Element)) return
      if (!t.closest('.react-flow__pane')) return
      onPaneClick(ev as unknown as React.MouseEvent)
    }
    document.addEventListener('click', handler, true)
    return () => document.removeEventListener('click', handler, true)
  }, [onPaneClick])
  return null
}

function FlowWorkspace({
  events,
  tree,
  expandedOf,
  toggleExpanded,
  toggleCrosscutBranch,
  resolvedBlocking,
  cancelRev,
}: {
  events: readonly UiEvent[]
  tree: ReturnType<typeof ingestEvents>
  expandedOf: (id: number) => boolean
  toggleExpanded: (id: number) => void
  toggleCrosscutBranch: (b: number | null, additive: boolean) => void
  resolvedBlocking: ReadonlySet<number>
  cancelRev: number
}) {
  const [tilt, setTilt] = useState({ rx: 0, ry: 0 })
  const [rfNodes, setRfNodes] = useState<BranchRfNode[]>([])

  const onNodesChange: OnNodesChange<BranchRfNode> = useCallback((changes) => {
    setRfNodes((nds) => applyNodeChanges(changes, nds))
  }, [])

  return (
    <div
      className="hick-flow-surface relative h-[calc(100vh-200px)] min-h-[360px] w-full"
      style={{ perspective: '1200px' }}
    >
      <div
        className="h-full w-full"
        style={{
          transform: `rotateX(${tilt.rx}deg) rotateY(${tilt.ry}deg)`,
          transformStyle: 'preserve-3d',
        }}
      >
        <ReactFlow
          nodes={rfNodes}
          onNodesChange={onNodesChange}
          nodeTypes={nodeTypes}
          fitView={false}
          className="!bg-slate-950"
          style={{ width: '100%', height: '100%' }}
          proOptions={{ hideAttribution: true }}
          panOnDrag={false}
          panOnScroll={false}
          zoomOnScroll={false}
          zoomOnPinch={false}
          zoomOnDoubleClick={false}
          preventScrolling
          nodesDraggable={false}
          nodesConnectable={false}
          elementsSelectable={false}
          minZoom={0.18}
          maxZoom={1.2}
        >
          <FlowLogic
            events={events}
            tree={tree}
            expandedOf={expandedOf}
            toggleExpanded={toggleExpanded}
            toggleCrosscutBranch={toggleCrosscutBranch}
            resolvedBlocking={resolvedBlocking}
            setRfNodes={setRfNodes}
            onTilt={(rx, ry) => setTilt({ rx, ry })}
            cancelRev={cancelRev}
          />
        </ReactFlow>
      </div>
    </div>
  )
}

export function ConversationCanvas({
  events,
  onBlockingLockedChange,
}: {
  events: readonly UiEvent[]
  onBlockingLockedChange?: (locked: boolean) => void
}) {
  const tree = useMemo(() => ingestEvents(events), [events])
  const [expandedMap, setExpandedMap] = useState<Map<number, boolean>>(() => new Map())
  const [crosscut, setCrosscut] = useState<Set<number | null>>(() => new Set())
  const [resolvedBlocking, setResolvedBlocking] = useState<Set<number>>(() => new Set())
  const [cancelRev, setCancelRev] = useState(0)

  const expandedOf = useCallback(
    (id: number) => {
      const n = tree.nodes.get(id)
      const stored = expandedMap.get(id)
      if (stored !== undefined) return stored
      if (n?.kind === 'thinking' || n?.kind === 'action' || n?.kind === 'mcp_pair') return !n.collapsed_by_default
      return true
    },
    [expandedMap, tree.nodes],
  )

  const toggleExpanded = useCallback((id: number) => {
    setExpandedMap((prev) => {
      const next = new Map(prev)
      const n = tree.nodes.get(id)
      const prevVal =
        prev.get(id) ??
        (n?.kind === 'thinking' || n?.kind === 'action' || n?.kind === 'mcp_pair'
          ? !n.collapsed_by_default
          : true)
      next.set(id, !prevVal)
      return next
    })
  }, [tree.nodes])

  const collapseAllChips = useCallback(() => {
    setExpandedMap((prev) => {
      const next = new Map(prev)
      for (const n of tree.nodes.values()) {
        if (n.kind === 'thinking' || n.kind === 'action' || n.kind === 'mcp_pair') next.set(n.id, false)
      }
      return next
    })
  }, [tree.nodes])

  const expandAllChips = useCallback(() => {
    setExpandedMap((prev) => {
      const next = new Map(prev)
      for (const n of tree.nodes.values()) {
        if (n.kind === 'thinking' || n.kind === 'action' || n.kind === 'mcp_pair') next.set(n.id, true)
      }
      return next
    })
  }, [tree.nodes])

  const toggleCrosscutBranch = useCallback((b: number | null, additive: boolean) => {
    setCrosscut((prev) => {
      const next = additive ? new Set(prev) : new Set<number | null>()
      if (additive) {
        if (prev.has(b)) next.delete(b)
        else next.add(b)
      } else {
        next.add(b)
      }
      return next
    })
  }, [])

  const blockingLocked = useMemo(
    () => [...tree.nodes.values()].some((n) => isBlockingKind(n.kind) && !resolvedBlocking.has(n.id)),
    [resolvedBlocking, tree.nodes],
  )

  useEffect(() => {
    onBlockingLockedChange?.(blockingLocked)
  }, [blockingLocked, onBlockingLockedChange])

  const blockingNodesBanner = useMemo(() => {
    const pending: ConversationNodeState[] = []
    for (const n of tree.nodes.values()) {
      if (isBlockingKind(n.kind) && !resolvedBlocking.has(n.id)) pending.push(n)
    }
    return pending
  }, [resolvedBlocking, tree.nodes])

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {blockingNodesBanner.length > 0 && (
        <div className="border-b border-amber-600/40 bg-amber-950/80 px-4 py-2 text-center text-xs text-amber-100">
          Blocking approval or security prompt is pending; unlock input after resolving where you launched the agent.
          <button
            type="button"
            className="ml-3 rounded border border-amber-500/60 px-2 py-0.5 text-[10px] text-amber-50 hover:bg-amber-900/50"
            onClick={() =>
              setResolvedBlocking((prev) => {
                const n = new Set(prev)
                blockingNodesBanner.forEach((b) => n.add(b.id))
                return n
              })
            }
          >
            Mark read (local)
          </button>
        </div>
      )}

      {events.length === 0 && (
        <div className="border-b border-slate-800 px-4 py-6 text-center text-sm text-slate-500">
          Waiting for session events…
        </div>
      )}

      <div className="flex items-center gap-2 border-b border-slate-800 bg-slate-900/90 px-3 py-2">
        <button
          type="button"
          className="rounded border border-slate-600 px-2 py-1 text-[11px] text-slate-200 hover:bg-slate-800"
          onClick={collapseAllChips}
        >
          Collapse all
        </button>
        <button
          type="button"
          className="rounded border border-slate-600 px-2 py-1 text-[11px] text-slate-200 hover:bg-slate-800"
          onClick={expandAllChips}
        >
          Expand all
        </button>
        <button
          type="button"
          className="rounded border border-slate-600 px-2 py-1 text-[11px] text-slate-200 hover:bg-slate-800"
          onClick={() => setCancelRev((s) => s + 1)}
        >
          Cancel motion
        </button>
        <span className="text-[10px] text-slate-500">
          Ctrl+click pane · crosscut strip · minimap jumps · pan/zoom disabled
        </span>
      </div>

      <ReactFlowProvider>
        <FlowWorkspace
          events={events}
          tree={tree}
          expandedOf={expandedOf}
          toggleExpanded={toggleExpanded}
          toggleCrosscutBranch={toggleCrosscutBranch}
          resolvedBlocking={resolvedBlocking}
          cancelRev={cancelRev}
        />
      </ReactFlowProvider>

      {crosscut.size > 0 && (
        <CrosscutStrip
          branches={[...crosscut]}
          tree={tree}
          expandedOf={expandedOf}
          toggleExpanded={toggleExpanded}
          onClose={(b) =>
            setCrosscut((prev) => {
              const n = new Set(prev)
              n.delete(b)
              return n
            })
          }
        />
      )}
    </div>
  )
}

function CrosscutStrip({
  branches,
  tree,
  expandedOf,
  toggleExpanded,
  onClose,
}: {
  branches: (number | null)[]
  tree: ReturnType<typeof ingestEvents>
  expandedOf: (id: number) => boolean
  toggleExpanded: (id: number) => void
  onClose: (b: number | null) => void
}) {
  return (
    <div className="border-t border-violet-700/40 bg-slate-950/95 px-2 py-2">
      <div className="mb-2 flex items-center justify-between px-2">
        <span className="text-[11px] font-semibold uppercase tracking-wide text-violet-300">
          Crosscut · {branches.length} panel(s) · ⚡ separators mark discontinuities between canvases
        </span>
      </div>
      <div className="flex max-h-[55vh] gap-0 overflow-x-auto">
        {branches.map((br, idx) => {
          const nodeIds = orderedBranchNodes(tree.nodes, tree.sessionRootId, br, tree.children)
          const sep = buildTurnSepLabels(nodeIds, tree.nodes)
          return (
            <div key={branchSlug(br)} className="flex min-w-[340px] shrink-0 items-stretch">
              {idx > 0 && (
                <div className="flex w-10 shrink-0 flex-col items-center justify-center border-x border-dashed border-violet-600/40 bg-slate-900/40 text-[10px] text-violet-400">
                  <span className="rotate-90 whitespace-nowrap">⚡</span>
                </div>
              )}
              <div className="relative flex min-w-[320px] flex-1 flex-col">
                <button
                  type="button"
                  className="absolute right-5 top-10 z-10 rounded bg-slate-800 px-2 py-0.5 text-[10px] text-slate-300 hover:bg-slate-700"
                  onClick={() => onClose(br)}
                >
                  × panel
                </button>
                <BranchColumnBody
                  branchKey={br}
                  nodeIds={nodeIds}
                  nodes={tree.nodes}
                  expandedOf={expandedOf}
                  toggleExpanded={toggleExpanded}
                  ambient={false}
                  turnSepLabels={sep}
                  columnOpacity={1}
                />
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
