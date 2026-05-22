import { useState } from 'react'

interface Props {
  nodeId: number
  text: string
  streaming: boolean
  onBranch?: (nodeId: number) => void
}

export function AssistantNode({ nodeId, text, streaming, onBranch }: Props) {
  const [hovered, setHovered] = useState(false)
  return (
    <div
      className="flex justify-start py-2 pl-4"
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      <div className="relative flex items-start gap-1">
        <div
          className="relative max-w-[70%] rounded-[18px] border border-slate-700 bg-slate-800 px-4 py-3 text-sm text-slate-100"
          style={{ borderRadius: '18px' }}
        >
          <span className="whitespace-pre-wrap break-words">{text}</span>
          {streaming && (
            <span className="ml-0.5 inline-block h-4 w-0.5 animate-pulse bg-slate-300" aria-label="typing" />
          )}
          {/* Left-side tail */}
          <svg
            width="18"
            height="14"
            viewBox="0 0 18 14"
            className="absolute -bottom-3 left-4 text-slate-800"
            aria-hidden="true"
          >
            <path
              d="M 0 0 C 4 0, 8 4, 6 12 C 10 8, 14 4, 18 0 Z"
              fill="currentColor"
            />
          </svg>
        </div>
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
      </div>
    </div>
  )
}
