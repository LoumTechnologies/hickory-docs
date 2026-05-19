interface Props {
  nodeId: number
  isCollapsed: boolean
  onToggle: () => void
}

export function ThinkingNode({ isCollapsed, onToggle }: Props) {
  return (
    <div className="flex justify-start py-1 pl-4">
      <button
        type="button"
        className="cursor-pointer text-left"
        onClick={onToggle}
        aria-expanded={!isCollapsed}
      >
        {isCollapsed ? (
          // Collapsed: small cloud shape with pulsing dots
          <div className="flex items-center gap-2 rounded-full border border-dotted border-slate-600 bg-slate-800/50 px-4 py-2 text-slate-400">
            <span className="text-xs text-slate-500">thinking</span>
            <span className="flex gap-1">
              <span className="inline-block h-1.5 w-1.5 animate-pulse rounded-full bg-slate-400" style={{ animationDelay: '0ms' }} />
              <span className="inline-block h-1.5 w-1.5 animate-pulse rounded-full bg-slate-400" style={{ animationDelay: '150ms' }} />
              <span className="inline-block h-1.5 w-1.5 animate-pulse rounded-full bg-slate-400" style={{ animationDelay: '300ms' }} />
            </span>
          </div>
        ) : (
          // Expanded: cloud-like dotted border
          <div className="relative">
            <div className="rounded-[40%] border-2 border-dotted border-slate-600 bg-slate-800/30 px-4 py-3 text-xs text-slate-400 italic">
              Extended thinking (click to collapse)
            </div>
            {/* Ascending circles tail */}
            <div className="absolute -bottom-5 left-6 flex items-end gap-0.5">
              <div className="h-1.5 w-1.5 rounded-full border border-slate-600 bg-slate-800/30" />
              <div className="h-2.5 w-2.5 rounded-full border border-slate-600 bg-slate-800/30" />
              <div className="h-3.5 w-3.5 rounded-full border border-slate-600 bg-slate-800/30" />
            </div>
          </div>
        )}
      </button>
    </div>
  )
}
