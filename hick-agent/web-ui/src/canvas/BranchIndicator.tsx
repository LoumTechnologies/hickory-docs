import type { BranchId, BranchMeta } from './treeModel'

interface Props {
  activeBranchId: BranchId
  branches: BranchMeta[]
  navigateToBranch: (id: BranchId) => void
  onCtrlClick?: (id: BranchId) => void
}

export function BranchIndicator({ activeBranchId, branches, navigateToBranch, onCtrlClick }: Props) {
  const activeBranch = branches.find((b) => b.id === activeBranchId)
  if (!activeBranch) return null

  const activeCol = activeBranch.columnIndex

  // Find branches adjacent (colIndex = activeCol ± 1 ... or parent/child)
  const leftBranches = branches.filter((b) => b.id !== activeBranchId && b.columnIndex < activeCol)
  const rightBranches = branches.filter((b) => b.id !== activeBranchId && b.columnIndex > activeCol)

  // Show the closest one on each side
  const leftNearest = leftBranches.sort((a, b) => b.columnIndex - a.columnIndex)[0]
  const rightNearest = rightBranches.sort((a, b) => a.columnIndex - b.columnIndex)[0]

  return (
    <>
      {leftNearest && (
        <button
          type="button"
          className="fixed left-0 top-1/2 z-50 flex -translate-y-1/2 flex-col items-center gap-1 rounded-r-lg bg-slate-800/90 px-2 py-4 text-xs text-slate-300 shadow-lg backdrop-blur hover:bg-slate-700/90 hover:text-white"
          style={{ transform: 'translateY(-50%)' }}
          onClick={(e) => {
            if (e.ctrlKey && onCtrlClick) {
              onCtrlClick(leftNearest.id)
            } else {
              navigateToBranch(leftNearest.id)
            }
          }}
          title={`Go to: ${leftNearest.label}`}
        >
          <span>◀</span>
          <span
            className="max-w-[60px] overflow-hidden text-ellipsis whitespace-nowrap"
            style={{ writingMode: 'vertical-rl', textOrientation: 'mixed' }}
          >
            {leftNearest.label}
          </span>
        </button>
      )}
      {rightNearest && (
        <button
          type="button"
          className="fixed right-0 top-1/2 z-50 flex -translate-y-1/2 flex-col items-center gap-1 rounded-l-lg bg-slate-800/90 px-2 py-4 text-xs text-slate-300 shadow-lg backdrop-blur hover:bg-slate-700/90 hover:text-white"
          style={{ transform: 'translateY(-50%)' }}
          onClick={(e) => {
            if (e.ctrlKey && onCtrlClick) {
              onCtrlClick(rightNearest.id)
            } else {
              navigateToBranch(rightNearest.id)
            }
          }}
          title={`Go to: ${rightNearest.label}`}
        >
          <span>▶</span>
          <span
            className="max-w-[60px] overflow-hidden text-ellipsis whitespace-nowrap"
            style={{ writingMode: 'vertical-rl', textOrientation: 'mixed' }}
          >
            {rightNearest.label}
          </span>
        </button>
      )}
    </>
  )
}
