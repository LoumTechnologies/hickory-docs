import type { BranchId } from '../canvas/treeModel'

interface Props {
  label: string
  targetBranchId: BranchId
  navigateToBranch: (id: BranchId) => void
}

export function BranchPointNode({ label, targetBranchId, navigateToBranch }: Props) {
  return (
    <div className="mx-4 my-2 flex justify-center">
      <button
        type="button"
        className="flex items-center gap-2 rounded-full border border-emerald-500/40 bg-emerald-950/60 px-4 py-1.5 text-xs text-emerald-300 shadow hover:bg-emerald-900/60 hover:text-emerald-200"
        onClick={() => navigateToBranch(targetBranchId)}
        title={`Go to branch: ${label}`}
      >
        <span>⎇</span>
        <span>Branch from here → {label}</span>
      </button>
    </div>
  )
}
