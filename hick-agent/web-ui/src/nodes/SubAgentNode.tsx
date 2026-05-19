import type { BranchId } from '../canvas/treeModel'

interface Props {
  label: string
  targetBranchId: BranchId
  navigateToBranch: (id: BranchId) => void
}

export function SubAgentNode({ label, targetBranchId, navigateToBranch }: Props) {
  return (
    <div className="mx-4 my-2 flex justify-center">
      <button
        type="button"
        className="flex items-center gap-2 rounded-full border border-indigo-500/40 bg-indigo-950/60 px-4 py-1.5 text-xs text-indigo-300 shadow hover:bg-indigo-900/60 hover:text-indigo-200"
        onClick={() => navigateToBranch(targetBranchId)}
        title={`Open sub-agent: ${label}`}
      >
        <span>{label}</span>
        <span>→</span>
      </button>
    </div>
  )
}
