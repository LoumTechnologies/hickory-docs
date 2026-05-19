import type { UiEvent } from '../types'

export type BranchId = number // 0 = main

export interface BranchMeta {
  id: BranchId
  label: string
  parentBranchId: BranchId | null
  columnIndex: number // negative = left, 0 = center, positive = right
  forkNodeId: number | null // node_id of the event that created this branch
  kind: 'main' | 'sub_agent' | 'branch'
}

export type DisplayItem =
  | { kind: 'user_turn'; nodeId: number; text: string }
  | { kind: 'thinking'; nodeId: number; text: string; collapsedByDefault: boolean }
  | { kind: 'assistant'; nodeId: number; text: string; streaming: boolean }
  | {
      kind: 'action_pair'
      startId: number
      endId: number | null
      lang: string
      code: string
      stdout: string
      stderr: string
      exitCode: number | null
      collapsedByDefault: boolean
    }
  | {
      kind: 'mcp_pair'
      callId: number
      responseId: number | null
      server: string
      method: string
      params: unknown
      result: unknown
      collapsedByDefault: boolean
    }
  | { kind: 'info'; nodeId: number; text: string }
  | { kind: 'error'; nodeId: number; text: string }
  | { kind: 'security_violation'; nodeId: number; description: string }
  | { kind: 'network_approval'; nodeId: number; host: string; suggestions: string[] }
  | { kind: 'adaptive_card'; nodeId: number; json: string }
  | { kind: 'sub_agent_spawned'; nodeId: number; label: string; targetBranchId: BranchId }
  | { kind: 'branch_point'; nodeId: number; forkNodeId: number; label: string; targetBranchId: BranchId }
  | { kind: 'done'; nodeId: number }
  | { kind: 'interrupted'; nodeId: number }

export interface TreeModel {
  branches: BranchMeta[]
  itemsByBranch: Map<BranchId, DisplayItem[]>
  latestBranchId: BranchId
}

// Assign column indices: main=0, first sub=+1, second=-1, third=+2, ...
function nextColumnIndex(usedCount: number): number {
  const n = usedCount + 1 // 1-based sub-agent index
  const half = Math.ceil(n / 2)
  return n % 2 === 1 ? half : -half
}

export function buildTreeModel(events: UiEvent[]): TreeModel {
  const branches: BranchMeta[] = [
    { id: 0, label: 'Main', parentBranchId: null, columnIndex: 0, forkNodeId: null, kind: 'main' },
  ]
  const itemsByBranch = new Map<BranchId, DisplayItem[]>([[0, []]])

  // Stack of branch ids; current branch is top
  const branchStack: BranchId[] = [0]
  // Next branch id counter
  let nextBranchId = 1
  // Count of sub-agents created so far (for column assignment)
  let subAgentCount = 0

  // Pending unpaired action_starts per branch: branch -> list of startIds
  const pendingActions = new Map<BranchId, number[]>()
  // Pending unpaired mcp_calls per branch: branch -> list of callIds
  const pendingMcpCalls = new Map<BranchId, number[]>()

  const currentBranch = () => branchStack[branchStack.length - 1]

  const getItems = (bid: BranchId): DisplayItem[] => {
    let arr = itemsByBranch.get(bid)
    if (!arr) {
      arr = []
      itemsByBranch.set(bid, arr)
    }
    return arr
  }

  for (const ev of events) {
    const bid = currentBranch()
    const items = getItems(bid)

    switch (ev.type) {
      case 'session_started':
        // No display item — session metadata shown in sidebar
        break

      case 'user_turn':
        items.push({ kind: 'user_turn', nodeId: ev.node_id, text: ev.text })
        break

      case 'thinking':
        items.push({ kind: 'thinking', nodeId: ev.node_id, text: ev.text ?? '', collapsedByDefault: true })
        break

      case 'token': {
        // Find the most recent streaming assistant item in this branch
        for (let i = items.length - 1; i >= 0; i--) {
          const item = items[i]
          if (item.kind === 'assistant' && item.streaming) {
            items[i] = { ...item, text: item.text + ev.text }
            break
          }
        }
        // If no streaming item exists yet, create one
        const hasStreaming = items.some((it) => it.kind === 'assistant' && it.streaming)
        if (!hasStreaming) {
          items.push({ kind: 'assistant', nodeId: ev.node_id, text: ev.text, streaming: true })
        }
        break
      }

      case 'assistant_done': {
        // Finalize the streaming assistant item, or add one if missing
        let finalized = false
        for (let i = items.length - 1; i >= 0; i--) {
          const item = items[i]
          if (item.kind === 'assistant' && item.streaming) {
            items[i] = { ...item, text: ev.text, streaming: false }
            finalized = true
            break
          }
        }
        if (!finalized) {
          items.push({ kind: 'assistant', nodeId: ev.node_id, text: ev.text, streaming: false })
        }
        break
      }

      case 'action_start': {
        const item: DisplayItem = {
          kind: 'action_pair',
          startId: ev.node_id,
          endId: null,
          lang: ev.lang,
          code: ev.code,
          stdout: '',
          stderr: '',
          exitCode: null,
          collapsedByDefault: ev.collapsed_by_default,
        }
        items.push(item)
        const pending = pendingActions.get(bid) ?? []
        pending.push(ev.node_id)
        pendingActions.set(bid, pending)
        break
      }

      case 'action_end': {
        const pending = pendingActions.get(bid)
        if (pending && pending.length > 0) {
          const startId = pending.shift()!
          for (let i = items.length - 1; i >= 0; i--) {
            const item = items[i]
            if (item.kind === 'action_pair' && item.startId === startId && item.endId === null) {
              items[i] = {
                ...item,
                endId: ev.node_id,
                stdout: ev.stdout,
                stderr: ev.stderr,
                exitCode: ev.exit_code,
              }
              break
            }
          }
        }
        break
      }

      case 'mcp_call': {
        const item: DisplayItem = {
          kind: 'mcp_pair',
          callId: ev.node_id,
          responseId: null,
          server: ev.server,
          method: ev.method,
          params: null,
          result: null,
          collapsedByDefault: ev.collapsed_by_default,
        }
        items.push(item)
        const pending = pendingMcpCalls.get(bid) ?? []
        pending.push(ev.node_id)
        pendingMcpCalls.set(bid, pending)
        break
      }

      case 'mcp_response': {
        const pending = pendingMcpCalls.get(bid)
        if (pending && pending.length > 0) {
          const callId = pending.shift()!
          for (let i = items.length - 1; i >= 0; i--) {
            const item = items[i]
            if (item.kind === 'mcp_pair' && item.callId === callId && item.responseId === null) {
              items[i] = { ...item, responseId: ev.node_id }
              break
            }
          }
        }
        break
      }

      case 'sub_agent_spawned': {
        const newBranchId = nextBranchId++
        const colIdx = nextColumnIndex(subAgentCount++)
        branches.push({
          id: newBranchId,
          label: ev.label,
          parentBranchId: bid,
          columnIndex: colIdx,
          forkNodeId: ev.node_id,
          kind: 'sub_agent',
        })
        itemsByBranch.set(newBranchId, [])
        items.push({
          kind: 'sub_agent_spawned',
          nodeId: ev.node_id,
          label: ev.label,
          targetBranchId: newBranchId,
        })
        branchStack.push(newBranchId)
        break
      }

      case 'sub_agent_done':
        if (branchStack.length > 1) {
          branchStack.pop()
        }
        break

      case 'branch_started': {
        const newBranchId = nextBranchId++
        const colIdx = nextColumnIndex(subAgentCount++)
        branches.push({
          id: newBranchId,
          label: ev.label,
          parentBranchId: bid,
          columnIndex: colIdx,
          forkNodeId: ev.fork_node_id,
          kind: 'branch',
        })
        itemsByBranch.set(newBranchId, [])
        items.push({
          kind: 'branch_point',
          nodeId: ev.node_id,
          forkNodeId: ev.fork_node_id,
          label: ev.label,
          targetBranchId: newBranchId,
        })
        branchStack.push(newBranchId)
        break
      }

      case 'security_violation':
        items.push({ kind: 'security_violation', nodeId: ev.node_id, description: ev.description })
        break

      case 'network_approval':
        items.push({
          kind: 'network_approval',
          nodeId: ev.node_id,
          host: ev.host,
          suggestions: ev.suggestions,
        })
        break

      case 'adaptive_card':
        items.push({ kind: 'adaptive_card', nodeId: ev.node_id, json: ev.json })
        break

      case 'info':
        items.push({ kind: 'info', nodeId: ev.node_id, text: ev.text })
        break

      case 'error':
        items.push({ kind: 'error', nodeId: ev.node_id, text: ev.text })
        break

      case 'done':
        items.push({ kind: 'done', nodeId: ev.node_id })
        break

      case 'interrupted':
        items.push({ kind: 'interrupted', nodeId: ev.node_id })
        break
    }
  }

  return { branches, itemsByBranch, latestBranchId: currentBranch() }
}
