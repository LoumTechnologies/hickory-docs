/**
 * Incremental mirror of Rust `ConversationTree` semantics plus branch-column helpers.
 */

import type { UiEvent } from '../types'

export type UiKind =
  | 'session'
  | 'user_turn'
  | 'thinking'
  | 'assistant'
  | 'action'
  | 'sub_agent'
  | 'mcp_pair'
  | 'security_violation'
  | 'network_approval'
  | 'adaptive_card'
  | 'info'
  | 'error'
  | 'done'
  | 'interrupted'

export interface ConversationNodeState {
  id: number
  parent_id: number | null
  kind: UiKind
  collapsed_by_default: boolean
  /** Streaming / consolidated assistant text */
  text?: string
  /** Thinking placeholder until backend exposes transcripts */
  thinkingNote?: string
  session?: {
    model: string | null
    sandbox_image: string | null
    provider_info: string | null
  }
  action?: {
    lang: string
    code: string
    stdout: string
    stderr: string
    exit_code: number | null
  }
  sub_agent?: { label: string }
  mcp?: {
    server: string
    method: string
    /** Synthetic pairing — wire lacks structured bodies */
    requestJson: Record<string, unknown>
    responseJson: Record<string, unknown>
  }
  security?: { description: string }
  network?: { host: string; suggestions: string[] }
  adaptive_card_json?: string
  info_text?: string
  error_text?: string
}

/** Which lateral branch column (`null` = main thread under session root). */
export function branchColumnOf(
  nodes: ReadonlyMap<number, ConversationNodeState>,
  nodeId: number,
): number | null {
  const node = nodes.get(nodeId)
  if (!node) return null
  if (node.kind === 'sub_agent') {
    if (node.parent_id === null) return null
    return branchColumnOf(nodes, node.parent_id)
  }
  let cur = node.parent_id
  while (cur !== null) {
    const n = nodes.get(cur)
    if (!n) break
    if (n.kind === 'sub_agent') return cur
    cur = n.parent_id
  }
  return null
}

export interface ChildrenMap {
  get(parentKey: number | 'root'): readonly number[] | undefined
}

/** Preserve insertion order as events arrive (matches Rust `.push` semantics). */
export function buildChildrenMapFromEdges(edges: readonly { parentKey: number | 'root'; childId: number }[]): Map<number | 'root', number[]> {
  const m = new Map<number | 'root', number[]>()
  for (const { parentKey, childId } of edges) {
    const arr = m.get(parentKey)
    if (arr) arr.push(childId)
    else m.set(parentKey, [childId])
  }
  return m
}

/** DFS emit children under session root where branchColumn matches `branch`. */
export function orderedBranchNodes(
  nodes: ReadonlyMap<number, ConversationNodeState>,
  sessionRootId: number | null,
  branch: number | null,
  children: ReadonlyMap<number | 'root', readonly number[]>,
): number[] {
  if (sessionRootId === null) return []

  const out: number[] = []
  const walk = (nid: number) => {
    if (branchColumnOf(nodes, nid) === branch) {
      out.push(nid)
    }
    const kids = children.get(nid)
    if (!kids) return
    for (const c of kids) walk(c)
  }

  walk(sessionRootId)
  return out
}

/** Collect distinct branch columns (excluding main null, excluding duplicates). */
export function branchColumnsInOrder(
  nodes: ReadonlyMap<number, ConversationNodeState>,
  sessionRootId: number | null,
  children: ReadonlyMap<number | 'root', readonly number[]>,
): (number | null)[] {
  const seen = new Set<number | string>()
  const cols: (number | null)[] = []

  const pushCol = (c: number | null) => {
    const key = c === null ? 'main' : c
    if (seen.has(key)) return
    seen.add(key)
    cols.push(c)
  }

  pushCol(null)

  const dfsBranches = (nid: number) => {
    const kids = children.get(nid)
    if (!kids) return
    for (const c of kids) {
      const n = nodes.get(c)
      if (n?.kind === 'sub_agent') pushCol(c)
      dfsBranches(c)
    }
  }

  if (sessionRootId !== null) dfsBranches(sessionRootId)

  for (const n of nodes.values()) {
    if (n.kind === 'sub_agent') pushCol(n.id)
  }

  return cols
}

export interface TreeAccum {
  nodes: Map<number, ConversationNodeState>
  sessionRootId: number | null
  children: Map<number | 'root', number[]>
}

export function ingestEvents(events: readonly UiEvent[]): TreeAccum {
  const nodes = new Map<number, ConversationNodeState>()
  let sessionRootId: number | null = null
  const childEdges: { parentKey: number | 'root'; childId: number }[] = []

  const insert = (
    id: number,
    parent_id: number | null,
    partial: Omit<ConversationNodeState, 'id' | 'parent_id'>,
  ) => {
    nodes.set(id, { id, parent_id, ...partial })
    const parentKey = parent_id === null ? 'root' : parent_id
    childEdges.push({ parentKey, childId: id })
  }

  const appendAssistantText = (id: number, delta: string) => {
    const existing = nodes.get(id)
    if (existing) {
      existing.text = (existing.text ?? '') + delta
      existing.kind = 'assistant'
      return
    }
    insert(id, null, {
      kind: 'assistant',
      collapsed_by_default: false,
      text: delta,
    })
  }

  const setAssistantFull = (id: number, text: string) => {
    const existing = nodes.get(id)
    if (existing) {
      existing.text = text
      existing.kind = 'assistant'
      return
    }
    insert(id, null, {
      kind: 'assistant',
      collapsed_by_default: false,
      text,
    })
  }

  const pendingMcpCall: number[] = []

  for (const ev of events) {
    switch (ev.type) {
      case 'session_started':
        sessionRootId = ev.node_id
        insert(ev.node_id, null, {
          kind: 'session',
          collapsed_by_default: false,
          session: {
            model: ev.model,
            sandbox_image: ev.sandbox_image,
            provider_info: ev.provider_info,
          },
        })
        break
      case 'user_turn':
        insert(ev.node_id, ev.parent_id, {
          kind: 'user_turn',
          collapsed_by_default: false,
          text: ev.text,
        })
        break
      case 'thinking':
        insert(ev.node_id, ev.parent_id, {
          kind: 'thinking',
          collapsed_by_default: false,
          thinkingNote: '',
        })
        break
      case 'token':
        appendAssistantText(ev.node_id, ev.text)
        break
      case 'assistant_done':
        setAssistantFull(ev.node_id, ev.text)
        break
      case 'action_start':
        insert(ev.node_id, ev.parent_id, {
          kind: 'action',
          collapsed_by_default: ev.collapsed_by_default,
          action: {
            lang: ev.lang,
            code: ev.code,
            stdout: '',
            stderr: '',
            exit_code: null,
          },
        })
        break
      case 'action_end': {
        const node = nodes.get(ev.node_id)
        if (node?.action) {
          node.action.stdout = ev.stdout
          node.action.stderr = ev.stderr
          node.action.exit_code = ev.exit_code
          node.collapsed_by_default = ev.collapsed_by_default
        } else {
          insert(ev.node_id, null, {
            kind: 'action',
            collapsed_by_default: ev.collapsed_by_default,
            action: {
              lang: '',
              code: '',
              stdout: ev.stdout,
              stderr: ev.stderr,
              exit_code: ev.exit_code,
            },
          })
        }
        break
      }
      case 'sub_agent_spawned':
        insert(ev.node_id, ev.parent_id, {
          kind: 'sub_agent',
          collapsed_by_default: false,
          sub_agent: { label: ev.label },
        })
        break
      case 'sub_agent_done':
        break
      case 'mcp_call':
        pendingMcpCall.push(ev.node_id)
        insert(ev.node_id, ev.parent_id, {
          kind: 'mcp_pair',
          collapsed_by_default: ev.collapsed_by_default,
          mcp: {
            server: ev.server,
            method: ev.method,
            requestJson: {
              server: ev.server,
              method: ev.method,
              parent_id: ev.parent_id,
              note: 'Structured MCP params are not exposed on the wire yet.',
            },
            responseJson: { pending: true },
          },
        })
        break
      case 'mcp_response': {
        /** Attach response to latest unmatched call with matching server/method when possible */
        let targetId: number | undefined
        for (let i = pendingMcpCall.length - 1; i >= 0; i--) {
          const cid = pendingMcpCall[i]
          const cn = nodes.get(cid)
          if (cn?.mcp?.server === ev.server && cn?.mcp.method === ev.method) {
            targetId = cid
            pendingMcpCall.splice(i, 1)
            break
          }
        }
        const attachTo = targetId ?? pendingMcpCall.pop()
        const holder = attachTo !== undefined ? nodes.get(attachTo) : undefined
        if (holder?.mcp) {
          holder.mcp.responseJson = {
            server: ev.server,
            method: ev.method,
            collapsed_by_default: ev.collapsed_by_default,
            note: 'Structured MCP response body is not exposed on the wire yet.',
          }
          holder.collapsed_by_default = holder.collapsed_by_default || ev.collapsed_by_default
        } else {
          insert(ev.node_id, null, {
            kind: 'mcp_pair',
            collapsed_by_default: ev.collapsed_by_default,
            mcp: {
              server: ev.server,
              method: ev.method,
              requestJson: { inferred: false },
              responseJson: {
                server: ev.server,
                method: ev.method,
                note: 'Orphan MCP response — preceding call missing or mismatched.',
              },
            },
          })
        }
        break
      }
      case 'security_violation':
        insert(ev.node_id, null, {
          kind: 'security_violation',
          collapsed_by_default: false,
          security: { description: ev.description },
        })
        break
      case 'network_approval':
        insert(ev.node_id, null, {
          kind: 'network_approval',
          collapsed_by_default: false,
          network: { host: ev.host, suggestions: ev.suggestions },
        })
        break
      case 'adaptive_card':
        insert(ev.node_id, ev.parent_id, {
          kind: 'adaptive_card',
          collapsed_by_default: false,
          adaptive_card_json: ev.json,
        })
        break
      case 'info':
        insert(ev.node_id, null, {
          kind: 'info',
          collapsed_by_default: false,
          info_text: ev.text,
        })
        break
      case 'error':
        insert(ev.node_id, null, {
          kind: 'error',
          collapsed_by_default: false,
          error_text: ev.text,
        })
        break
      case 'done':
        insert(ev.node_id, null, {
          kind: 'done',
          collapsed_by_default: false,
        })
        break
      case 'interrupted':
        insert(ev.node_id, null, {
          kind: 'interrupted',
          collapsed_by_default: false,
        })
        break
    }
  }

  const children = buildChildrenMapFromEdges(childEdges)
  return { nodes, sessionRootId, children }
}

export function isBlockingKind(kind: UiKind): boolean {
  return kind === 'security_violation' || kind === 'network_approval'
}
