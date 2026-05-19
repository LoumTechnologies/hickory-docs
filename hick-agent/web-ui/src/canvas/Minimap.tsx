import type { BranchId, BranchMeta } from './treeModel'

interface MinimapProps {
  branches: BranchMeta[]
  activeBranchId: BranchId
  itemCountByBranch: Map<BranchId, number>
  navigateToBranch: (id: BranchId) => void
}

const MINI_COL_W = 48
const MINI_COL_H = 120
const MINI_GAP = 6
const MINI_PADDING = 8

export function Minimap({ branches, activeBranchId, itemCountByBranch, navigateToBranch }: MinimapProps) {
  if (branches.length <= 1) return null

  const sorted = [...branches].sort((a, b) => a.columnIndex - b.columnIndex)
  const totalW = sorted.length * MINI_COL_W + (sorted.length - 1) * MINI_GAP + MINI_PADDING * 2
  const totalH = MINI_COL_H + MINI_PADDING * 2

  const maxItems = Math.max(...[...itemCountByBranch.values()], 1)

  return (
    <div
      className="pointer-events-auto absolute bottom-4 right-4 z-30 rounded border border-slate-700 bg-slate-900/90 shadow-lg backdrop-blur"
      style={{ width: totalW, height: totalH }}
      role="navigation"
      aria-label="Branch minimap"
    >
      <div className="px-1 pt-1 text-[9px] font-semibold uppercase tracking-wide text-slate-500">
        Branches
      </div>
      <div
        className="flex items-end"
        style={{ gap: MINI_GAP, padding: `0 ${MINI_PADDING}px ${MINI_PADDING}px` }}
      >
        {sorted.map((branch) => {
          const isActive = branch.id === activeBranchId
          const count = itemCountByBranch.get(branch.id) ?? 0
          const fillH = Math.max(4, Math.round((count / maxItems) * (MINI_COL_H - 24)))

          return (
            <button
              key={branch.id}
              type="button"
              title={`Go to: ${branch.label}`}
              onClick={() => navigateToBranch(branch.id)}
              className={`flex flex-col items-center justify-end rounded transition-colors ${
                isActive
                  ? 'ring-1 ring-sky-400'
                  : 'hover:bg-slate-700/60'
              }`}
              style={{ width: MINI_COL_W, height: MINI_COL_H - 8 }}
            >
              {/* Item density bar */}
              <div
                className={`w-3/4 rounded-sm transition-all ${isActive ? 'bg-sky-500/60' : 'bg-slate-600/60'}`}
                style={{ height: fillH }}
              />
              {/* Label */}
              <span
                className={`mt-1 max-w-full overflow-hidden text-ellipsis whitespace-nowrap px-0.5 text-[9px] leading-tight ${
                  isActive ? 'text-sky-300' : 'text-slate-400'
                }`}
                style={{ maxWidth: MINI_COL_W - 4 }}
              >
                {branch.label}
              </span>
            </button>
          )
        })}
      </div>
    </div>
  )
}
