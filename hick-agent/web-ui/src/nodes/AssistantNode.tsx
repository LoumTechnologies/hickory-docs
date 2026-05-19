interface Props {
  text: string
  streaming: boolean
}

export function AssistantNode({ text, streaming }: Props) {
  return (
    <div className="flex justify-start py-2 pl-4">
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
            d="M 18 0 C 14 0, 10 4, 12 12 C 8 8, 4 4, 0 0 Z"
            fill="currentColor"
          />
        </svg>
      </div>
    </div>
  )
}
