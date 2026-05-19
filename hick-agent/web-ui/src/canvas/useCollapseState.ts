import { useState, useCallback, useEffect } from 'react'
import type { TreeModel } from './treeModel'

export interface CollapseState {
  isCollapsed: (nodeId: number) => boolean
  toggle: (nodeId: number) => void
  collapseAll: () => void
  expandAll: () => void
}

export function useCollapseState(model: TreeModel): CollapseState {
  const [collapsed, setCollapsed] = useState<Map<number, boolean>>(new Map())

  // Seed initial state from model items collapsedByDefault
  useEffect(() => {
    setCollapsed((prev) => {
      const next = new Map(prev)
      for (const items of model.itemsByBranch.values()) {
        for (const item of items) {
          if ('collapsedByDefault' in item) {
            const nodeId =
              item.kind === 'action_pair'
                ? item.startId
                : item.kind === 'mcp_pair'
                  ? item.callId
                  : item.nodeId
            if (!prev.has(nodeId)) {
              next.set(nodeId, item.collapsedByDefault)
            }
          }
        }
      }
      return next
    })
  }, [model])

  const isCollapsed = useCallback(
    (nodeId: number): boolean => {
      return collapsed.get(nodeId) ?? false
    },
    [collapsed],
  )

  const toggle = useCallback((nodeId: number) => {
    setCollapsed((prev) => {
      const next = new Map(prev)
      next.set(nodeId, !prev.get(nodeId))
      return next
    })
  }, [])

  const collapseAll = useCallback(() => {
    setCollapsed((prev) => {
      const next = new Map(prev)
      for (const items of model.itemsByBranch.values()) {
        for (const item of items) {
          if (item.kind === 'thinking' || item.kind === 'action_pair' || item.kind === 'mcp_pair') {
            const nodeId = item.kind === 'action_pair' ? item.startId : item.kind === 'mcp_pair' ? item.callId : item.nodeId
            next.set(nodeId, true)
          }
        }
      }
      return next
    })
  }, [model])

  const expandAll = useCallback(() => {
    setCollapsed((prev) => {
      const next = new Map(prev)
      for (const items of model.itemsByBranch.values()) {
        for (const item of items) {
          if (item.kind === 'thinking' || item.kind === 'action_pair' || item.kind === 'mcp_pair') {
            const nodeId = item.kind === 'action_pair' ? item.startId : item.kind === 'mcp_pair' ? item.callId : item.nodeId
            next.set(nodeId, false)
          }
        }
      }
      return next
    })
  }, [model])

  return { isCollapsed, toggle, collapseAll, expandAll }
}
