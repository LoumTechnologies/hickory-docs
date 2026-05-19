import { useState } from 'react'

interface Props {
  nodeId: number
  text: string
  onBranch?: (nodeId: number) => void
}

export function UserTurnNode({ nodeId, text, onBranch }: Props) {
  const [hovered, setHovered] = useState(false)
  return (
    <div
      className="flex justify-end py-2 pr-4"
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div className="relative flex items-start gap-1">
        {onBranch && hovered && (
          <button
            type="button"
            className="mt-1 rounded p-0.5 text-slate-500 hover:text-emerald-400"
            title="Branch from here"
            onClick={() => onBranch(nodeId)}
          >
            ⎇
          </button>
        )}
      <div
        className="relative max-w-[70%] rounded-[18px] border border-sky-500/40 bg-sky-900/80 px-4 py-3 text-sm text-sky-100"
        style={{ borderRadius: '18px' }}
      >
        <span className="whitespace-pre-wrap break-words">{text}</span>
        {/* Right-side tail */}
        <svg
          width="18"
          height="14"
          viewBox="0 0 18 14"
          className="absolute -bottom-3 right-4 text-sky-900/80"
          aria-hidden="true"
        >
          <path
            d="M 0 0 C 4 0, 8 4, 6 12 C 10 8, 14 4, 18 0 Z"
            fill="currentColor"
          />
        </svg>
      </div>
      </div>
    </div>
  )
}
