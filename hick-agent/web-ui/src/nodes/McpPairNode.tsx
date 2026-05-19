import { useState } from 'react'

interface Props {
  callId: number
  server: string
  method: string
  params: unknown
  result: unknown
  isCollapsed: boolean
  onToggle: () => void
}

function jsonPreview(obj: unknown, maxKeys = 3): string {
  if (obj === null || obj === undefined) return '—'
  if (typeof obj === 'string') return obj.slice(0, 80)
  if (typeof obj !== 'object') return String(obj)
  const entries = Object.entries(obj as Record<string, unknown>).slice(0, maxKeys)
  return entries.map(([k, v]) => `${k}: ${JSON.stringify(v)}`).join(', ')
}

export function McpPairNode({ server, method, params, result, isCollapsed, onToggle }: Props) {
  const [showFullJson, setShowFullJson] = useState(false)

  if (isCollapsed) {
    return (
      <div className="mx-4 my-1">
        <button
          type="button"
          className="flex w-full items-center gap-2 rounded border border-slate-700 bg-slate-800/40 px-3 py-1.5 text-left text-xs hover:bg-slate-700/40"
          onClick={onToggle}
        >
          <span className="shrink-0 text-slate-400">→</span>
          <span className="flex-1 truncate font-mono text-slate-300">
            {server}/{method}
          </span>
          <span className="shrink-0 text-green-400">200</span>
        </button>
      </div>
    )
  }

  return (
    <div className="mx-4 my-2">
      <div className="rounded border border-slate-600 text-xs overflow-hidden">
        <button
          type="button"
          className="flex w-full items-center justify-between border-b border-slate-600 bg-slate-800 px-3 py-1.5 text-left hover:bg-slate-700/60"
          onClick={onToggle}
        >
          <span className="font-mono font-semibold text-slate-300">
            → {server}/{method}
          </span>
          <span className="text-green-400">200</span>
        </button>

        {/* Params */}
        <div className="border-b border-slate-700 bg-slate-900/50 px-3 py-2">
          <div className="mb-1 text-slate-500">Request</div>
          <div className="font-mono text-slate-300">{jsonPreview(params)}</div>
        </div>

        {/* Result */}
        <div className="bg-slate-900/30 px-3 py-2">
          <div className="mb-1 text-slate-500">Response</div>
          <div className="font-mono text-slate-300">{jsonPreview(result)}</div>
        </div>

        {/* Full JSON toggle */}
        {(params !== null || result !== null) && (
          <div className="border-t border-slate-700 px-3 py-1">
            <button
              type="button"
              className="text-sky-400 hover:text-sky-300"
              onClick={(e) => {
                e.stopPropagation()
                setShowFullJson((v) => !v)
              }}
            >
              {showFullJson ? 'Hide' : 'View'} full JSON
            </button>
            {showFullJson && (
              <pre className="mt-2 max-h-64 overflow-auto rounded bg-slate-950 p-2 font-mono text-xs text-slate-200">
                {JSON.stringify({ params, result }, null, 2)}
              </pre>
            )}
          </div>
        )}
      </div>
    </div>
  )
}
