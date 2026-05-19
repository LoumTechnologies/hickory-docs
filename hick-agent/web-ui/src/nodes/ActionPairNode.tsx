interface Props {
  startId: number
  lang: string
  code: string
  stdout: string
  stderr: string
  exitCode: number | null
  isCollapsed: boolean
  onToggle: () => void
}

export function ActionPairNode({ lang, code, stdout, stderr, exitCode, isCollapsed, onToggle }: Props) {
  const exitBadgeClass =
    exitCode === null
      ? 'text-slate-400'
      : exitCode === 0
        ? 'text-green-400'
        : 'text-red-400'

  const exitLabel = exitCode === null ? '…' : String(exitCode)

  if (isCollapsed) {
    const codePreview = code.length > 60 ? code.slice(0, 60) + '…' : code
    return (
      <div className="mx-4 my-1">
        <button
          type="button"
          className="flex w-full items-center gap-2 rounded border border-slate-700 bg-slate-800/60 px-3 py-1.5 text-left text-xs hover:bg-slate-700/60"
          onClick={onToggle}
        >
          <span className="shrink-0 rounded bg-slate-700 px-1.5 py-0.5 font-mono text-slate-300">
            ▶ {lang}
          </span>
          <span className="min-w-0 flex-1 truncate font-mono text-slate-400">{codePreview}</span>
          <span className={`shrink-0 font-mono ${exitBadgeClass}`}>exit {exitLabel}</span>
        </button>
      </div>
    )
  }

  return (
    <div className="mx-4 my-2">
      <button
        type="button"
        className="w-full cursor-pointer text-left"
        onClick={onToggle}
      >
        <div className="rounded border border-slate-600 overflow-hidden">
          {/* Header */}
          <div className="flex items-center justify-between border-b border-slate-600 bg-slate-800 px-3 py-1.5 text-xs">
            <span className="font-mono font-semibold text-slate-300">─ [{lang}]</span>
            <span className={`font-mono ${exitBadgeClass}`}>exit {exitLabel}</span>
          </div>
          {/* Code */}
          <pre className="overflow-auto bg-slate-900 p-3 font-mono text-xs leading-relaxed text-slate-200 max-h-64">
            {code}
          </pre>
          {/* Output divider */}
          {(stdout || stderr) && (
            <>
              <div className="border-t border-slate-600 bg-slate-800 px-3 py-1 text-xs text-slate-500">
                output
              </div>
              <pre className="overflow-auto bg-slate-950 p-3 font-mono text-xs leading-relaxed text-slate-300 max-h-48">
                {stdout}
                {stderr && <span className="text-red-400">{stderr}</span>}
              </pre>
            </>
          )}
        </div>
      </button>
    </div>
  )
}
