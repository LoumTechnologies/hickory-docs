import type { BranchId, TreeModel } from './treeModel'

interface Props {
  model: TreeModel
  activeBranchId: BranchId
  navigateToBranch: (id: BranchId) => void
}

interface BlockingItem {
  branchId: BranchId
  nodeId: number
  kind: 'security_violation' | 'network_approval'
  label: string
}

function findBlockingItems(model: TreeModel): BlockingItem[] {
  const result: BlockingItem[] = []
  for (const [branchId, items] of model.itemsByBranch) {
    for (const item of items) {
      if (item.kind === 'security_violation') {
        result.push({ branchId, nodeId: item.nodeId, kind: 'security_violation', label: 'Security violation' })
      } else if (item.kind === 'network_approval') {
        result.push({ branchId, nodeId: item.nodeId, kind: 'network_approval', label: `Approve: ${item.host}` })
      }
    }
  }
  return result
}

export function HudIndicator({ model, activeBranchId, navigateToBranch }: Props) {
  const blockingItems = findBlockingItems(model)
  const offscreen = blockingItems.filter((item) => item.branchId !== activeBranchId)

  if (offscreen.length === 0) return null

  return (
    <div className="pointer-events-none fixed inset-0 z-40">
      {offscreen.map((item, idx) => (
        <button
          key={`${item.branchId}-${item.nodeId}`}
          type="button"
          className="pointer-events-auto absolute animate-pulse rounded-lg bg-amber-600/90 px-3 py-2 text-xs font-medium text-white shadow-lg hover:bg-amber-500/90"
          style={{ top: `${40 + idx * 48}px`, right: '16px' }}
          onClick={() => navigateToBranch(item.branchId)}
          title={item.label}
        >
          {item.kind === 'security_violation' ? '⚠' : '🌐'} {item.label}
        </button>
      ))}
    </div>
  )
}

export { findBlockingItems }
export type { BlockingItem }

// Re-export for use in ConversationCanvas
export function hasBlockingPrompts(model: TreeModel): boolean {
  for (const items of model.itemsByBranch.values()) {
    for (const item of items) {
      if (item.kind === 'security_violation' || item.kind === 'network_approval') {
        return true
      }
    }
  }
  return false
}
