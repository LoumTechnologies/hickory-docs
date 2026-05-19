import { AdaptiveCard } from 'adaptivecards'
/* eslint-disable react-refresh/only-export-components -- renderConversationRow is a layout helper for BranchColumnNode */
import type { ReactNode } from 'react'
import { useEffect, useRef, useState } from 'react'
import type { ConversationNodeState } from './treeModel'

function roughHighlight(code: string, lang: string): ReactNode {
  const trimmed = lang.trim().toLowerCase()
  const lines = code.split('\n')
  return (
    <pre className="m-0 overflow-x-auto whitespace-pre-wrap font-mono text-[11px] leading-snug text-slate-100">
      {lines.map((line, li) => {
        const pieces: ReactNode[] = []
        if (trimmed === 'rust' || trimmed === 'py' || trimmed === 'python' || trimmed === 'ts' || trimmed === 'tsx') {
          const r = /\b(fn|let|mut|pub|use|mod|impl|struct|enum|trait|async|await|return|if|else|for|while|match|self|import|from|def|class)\b/g
          let last = 0
          let m: RegExpExecArray | null
          let i = 0
          while ((m = r.exec(line)) !== null) {
            if (m.index > last) pieces.push(line.slice(last, m.index))
            pieces.push(
              <span key={`${li}-${i++}`} className="text-sky-300">
                {m[0]}
              </span>,
            )
            last = m.index + m[0].length
          }
          pieces.push(line.slice(last))
        } else {
          pieces.push(line)
        }
        return (
          <span key={li}>
            {pieces}
            {'\n'}
          </span>
        )
      })}
    </pre>
  )
}

const userTailPath = 'M 0 0 C 4 0, 8 4, 6 12 C 10 8, 14 4, 18 0 Z'

function UserBubble({ text, placeholder }: { text: string; placeholder?: boolean }) {
  return (
    <div className="relative ml-auto max-w-[min(92%,520px)] rounded-[18px] bg-cyan-600/85 px-3 py-2 pr-5 text-sm text-slate-950 shadow-md">
      <p className="m-0 whitespace-pre-wrap">{placeholder ? '…' : text}</p>
      <svg
        className="pointer-events-none absolute -bottom-[6px] right-1 h-4 w-[18px] text-cyan-600/85"
        viewBox="0 0 20 14"
        aria-hidden
      >
        <path d={userTailPath} fill="currentColor" />
      </svg>
    </div>
  )
}

const assistantTailPath = 'M 18 0 C 14 0, 10 4, 12 12 C 8 8, 4 4, 0 0 Z'

function AssistantBubble({ text, placeholder }: { text: string; placeholder?: boolean }) {
  return (
    <div className="relative mr-auto max-w-[min(92%,560px)] rounded-[18px] border border-slate-700/80 bg-slate-900/90 px-3 py-2 pl-5 text-sm text-slate-100 shadow-inner">
      <p className="m-0 whitespace-pre-wrap">{placeholder ? '…' : text}</p>
      <svg
        className="pointer-events-none absolute -bottom-[6px] left-0 h-4 w-[18px] text-slate-900/90"
        viewBox="0 0 20 14"
        aria-hidden
      >
        <path d={assistantTailPath} fill="currentColor" />
      </svg>
    </div>
  )
}

function ThoughtBubble({
  collapsed,
  onToggle,
  placeholder,
}: {
  collapsed: boolean
  onToggle: () => void
  placeholder?: boolean
}) {
  return (
    <button
      type="button"
      onClick={onToggle}
      className="relative mr-auto flex max-w-[min(92%,420px)] cursor-pointer rounded-[28px] border-2 border-dashed border-slate-500/70 bg-slate-900/60 px-3 py-2 text-left text-slate-200 shadow-md outline-none ring-sky-500/40 focus-visible:ring-2"
    >
      <div className="flex items-center gap-1 pl-1">
        <span className="inline-flex gap-0.5">
          <span className="inline-block h-1.5 w-1.5 animate-pulse rounded-full bg-slate-400" />
          <span className="inline-block h-2 w-2 animate-pulse rounded-full bg-slate-400 [animation-delay:120ms]" />
          <span className="inline-block h-2.5 w-2.5 animate-pulse rounded-full bg-slate-400 [animation-delay:240ms]" />
        </span>
        <span className="pl-2 text-xs text-slate-400">thinking</span>
      </div>
      {!collapsed && !placeholder && (
        <p className="mt-2 text-sm text-slate-200">LLM inference (no transcript on wire yet).</p>
      )}
      <div className="pointer-events-none absolute -bottom-1 left-6 flex items-end gap-0.5">
        <span className="inline-block h-1 w-1 rounded-full bg-slate-400" />
        <span className="inline-block h-1.5 w-1.5 rounded-full bg-slate-400" />
        <span className="inline-block h-2 w-2 rounded-full bg-slate-400" />
      </div>
    </button>
  )
}

function ActionChip({
  lang,
  code,
  stdout,
  stderr,
  exitCode,
  collapsed,
  onToggle,
  placeholder,
}: {
  lang: string
  code: string
  stdout: string
  stderr: string
  exitCode: number | null
  collapsed: boolean
  onToggle: () => void
  placeholder?: boolean
}) {
  const badge =
    exitCode === null || exitCode === undefined ? (
      <span className="rounded bg-slate-600 px-1.5 py-0.5 text-[10px] text-slate-200">…</span>
    ) : exitCode === 0 ? (
      <span className="rounded bg-emerald-900/70 px-1.5 py-0.5 text-[10px] text-emerald-200">exit {exitCode}</span>
    ) : (
      <span className="rounded bg-red-900/70 px-1.5 py-0.5 text-[10px] text-red-100">exit {exitCode}</span>
    )

  return (
    <button
      type="button"
      onClick={onToggle}
      className="w-full rounded-lg border border-slate-600/80 bg-slate-900/70 text-left shadow-sm outline-none ring-amber-500/30 focus-visible:ring-2"
    >
      <div className="flex items-center justify-between border-b border-slate-700/80 px-2 py-1">
        <span className="font-mono text-[11px] text-amber-200/90">[{lang || 'shell'}]</span>
        {placeholder ? null : badge}
      </div>
      {(!collapsed || placeholder) && (
        <div className="p-2">
          {placeholder ? (
            <div className="h-16 animate-pulse rounded bg-slate-800/80" />
          ) : (
            <>
              <div className="max-h-40 overflow-auto rounded border border-slate-800 bg-slate-950/80 p-2">
                {roughHighlight(code, lang)}
              </div>
              <div className="mt-2 max-h-40 overflow-auto rounded border border-t border-slate-800 bg-black/40 p-2 font-mono text-[11px] text-slate-300">
                {stdout || stderr ? (
                  <>
                    {stdout && <span className="whitespace-pre-wrap">{stdout}</span>}
                    {stderr && <span className="whitespace-pre-wrap text-red-300">{stderr}</span>}
                  </>
                ) : (
                  <span className="text-slate-500">(no output)</span>
                )}
              </div>
            </>
          )}
        </div>
      )}
      {collapsed && !placeholder && (
        <div className="px-2 py-1.5 font-mono text-[10px] text-slate-500">Script / action · click to expand</div>
      )}
    </button>
  )
}

function McpChip({
  server,
  method,
  req,
  res,
  collapsed,
  onToggle,
  placeholder,
}: {
  server: string
  method: string
  req: Record<string, unknown>
  res: Record<string, unknown>
  collapsed: boolean
  onToggle: () => void
  placeholder?: boolean
}) {
  const [showJson, setShowJson] = useState(false)
  return (
    <div className="w-full rounded-lg border border-indigo-800/70 bg-indigo-950/40 shadow-sm">
      <button
        type="button"
        onClick={onToggle}
        className="flex w-full items-center justify-between rounded-t-lg px-2 py-1 text-left outline-none ring-indigo-400/30 focus-visible:ring-2"
      >
        <span className="font-mono text-[11px] text-indigo-200">
          → {server} / {method}
        </span>
        <span className="text-[10px] text-slate-500">MCP</span>
      </button>
      {(!collapsed || placeholder) && (
        <div className="space-y-2 p-2">
          {placeholder ? (
            <div className="h-14 animate-pulse rounded bg-indigo-900/40" />
          ) : (
            <>
              <div className="rounded border border-indigo-900/60 bg-slate-950/60 p-2 font-mono text-[10px] text-slate-300">
                {Object.entries(req)
                  .slice(0, 6)
                  .map(([k, v]) => (
                    <div key={k}>
                      <span className="text-indigo-300">{k}:</span> {String(v)}
                    </div>
                  ))}
              </div>
              <div className="rounded border border-indigo-900/60 bg-slate-950/60 p-2 text-[11px] text-slate-200">
                {String((res as { note?: string }).note ?? JSON.stringify(res).slice(0, 160))}
              </div>
              <label className="flex cursor-pointer items-center gap-2 text-[11px] text-slate-400">
                <input type="checkbox" checked={showJson} onChange={(e) => setShowJson(e.target.checked)} />
                View full JSON
              </label>
              {showJson && (
                <pre className="max-h-40 overflow-auto rounded border border-slate-800 bg-black/50 p-2 font-mono text-[10px] text-slate-300">
                  {JSON.stringify({ request: req, response: res }, null, 2)}
                </pre>
              )}
            </>
          )}
        </div>
      )}
      {collapsed && !placeholder && (
        <div className="px-2 py-1 text-[10px] text-slate-500">Tool call · click to expand pair</div>
      )}
    </div>
  )
}

function BlockingCard({
  title,
  body,
  icon,
  pulsing,
}: {
  title: string
  body: string
  icon: string
  pulsing?: boolean
}) {
  return (
    <div
      className={`relative mr-auto max-w-[min(92%,480px)] rounded-xl border-2 px-3 py-2 text-left ${
        pulsing
          ? 'animate-pulse border-amber-400/70 bg-amber-950/50 shadow-[0_0_0_4px_rgba(251,191,36,0.12)]'
          : 'border-amber-600/40 bg-amber-950/30'
      }`}
    >
      <div className="text-sm font-semibold text-amber-100">
        <span className="mr-2">{icon}</span>
        {title}
      </div>
      <p className="mt-1 whitespace-pre-wrap text-xs text-amber-50/90">{body}</p>
    </div>
  )
}

function AdaptiveHost({ json }: { json: string }) {
  const elRef = useRef<HTMLDivElement | null>(null)
  useEffect(() => {
    const el = elRef.current
    if (!el) return
    el.innerHTML = ''
    try {
      const card = new AdaptiveCard()
      card.parse(JSON.parse(json))
      const rendered = card.render()
      if (rendered instanceof HTMLElement) el.appendChild(rendered)
    } catch {
      el.textContent = 'Invalid Adaptive Card JSON'
    }
  }, [json])
  return <div ref={elRef} className="ac-host rounded-lg border border-slate-700 bg-slate-900/80 p-2 text-sm text-slate-100" />
}

export function TurnSeparator({ label }: { label: string }) {
  return (
    <div className="flex items-center gap-2 py-2 text-[10px] text-slate-500">
      <div className="h-px flex-1 bg-slate-700/80" />
      <span>{label}</span>
      <div className="h-px flex-1 bg-slate-700/80" />
    </div>
  )
}

export function renderConversationRow(
  node: ConversationNodeState,
  opts: {
    expanded: boolean
    onToggle: (id: number) => void
    /** Heuristic separator before this row */
    showTurnSep?: string
    placeholder?: boolean
  },
): ReactNode {
  const { expanded, onToggle, showTurnSep, placeholder } = opts

  const chipCollapsed = !expanded

  const inner = (): ReactNode => {
    switch (node.kind) {
      case 'session':
        return (
          <div className="rounded border border-slate-700 bg-slate-900/60 p-2 text-xs text-slate-300">
            <div>Session started</div>
            {node.session && (
              <div className="mt-1 space-y-0.5 font-mono text-[10px] text-slate-400">
                <div>model: {node.session.model ?? '—'}</div>
                <div>sandbox: {node.session.sandbox_image ?? '—'}</div>
                <div>provider: {node.session.provider_info ?? '—'}</div>
              </div>
            )}
          </div>
        )
      case 'user_turn':
        return <UserBubble text={node.text ?? ''} placeholder={placeholder} />
      case 'assistant':
        return <AssistantBubble text={node.text ?? ''} placeholder={placeholder} />
      case 'thinking':
        return (
          <ThoughtBubble collapsed={chipCollapsed} onToggle={() => onToggle(node.id)} placeholder={placeholder} />
        )
      case 'action':
        return (
          <ActionChip
            lang={node.action?.lang ?? ''}
            code={node.action?.code ?? ''}
            stdout={node.action?.stdout ?? ''}
            stderr={node.action?.stderr ?? ''}
            exitCode={node.action?.exit_code ?? null}
            collapsed={chipCollapsed}
            onToggle={() => onToggle(node.id)}
            placeholder={placeholder}
          />
        )
      case 'mcp_pair':
        return (
          <McpChip
            server={node.mcp?.server ?? ''}
            method={node.mcp?.method ?? ''}
            req={(node.mcp?.requestJson ?? {}) as Record<string, unknown>}
            res={(node.mcp?.responseJson ?? {}) as Record<string, unknown>}
            collapsed={chipCollapsed}
            onToggle={() => onToggle(node.id)}
            placeholder={placeholder}
          />
        )
      case 'security_violation':
        return <BlockingCard title="Security violation" body={node.security?.description ?? ''} icon="⚠" pulsing />
      case 'network_approval':
        return (
          <BlockingCard
            title={`Network approval · ${node.network?.host ?? ''}`}
            body={(node.network?.suggestions ?? []).join(', ') || 'Awaiting approval'}
            icon="🌐"
            pulsing
          />
        )
      case 'adaptive_card':
        return placeholder ? (
          <div className="h-24 animate-pulse rounded-lg bg-slate-800/80" />
        ) : (
          <AdaptiveHost json={node.adaptive_card_json ?? '{}'} />
        )
      case 'info':
        return (
          <div className="flex justify-center py-1">
            <span className="inline-flex items-center gap-1 font-mono text-[10px] text-slate-500">
              <span>ℹ</span>
              {node.info_text}
            </span>
          </div>
        )
      case 'error':
        return (
          <div className="flex justify-center py-1">
            <span className="inline-flex items-center gap-1 font-mono text-[10px] text-red-400">
              <span>✕</span>
              {node.error_text}
            </span>
          </div>
        )
      case 'sub_agent':
        return (
          <div className="rounded border border-violet-700/60 bg-violet-950/30 px-2 py-1 text-center text-[11px] text-violet-200">
            Sub-agent · {node.sub_agent?.label}
          </div>
        )
      case 'done':
        return <div className="py-1 text-center font-mono text-[10px] text-slate-600">── done ──</div>
      case 'interrupted':
        return <div className="py-1 text-center font-mono text-[10px] text-amber-600">── interrupted ──</div>
      default:
        return null
    }
  }

  return (
    <div className="px-2">
      {showTurnSep && <TurnSeparator label={showTurnSep} />}
      <div className="relative flex min-h-[1px] gap-2">
        <div className="w-[2px] shrink-0 rounded-full bg-slate-700/60" aria-hidden />
        <div className="min-w-0 flex-1 py-1.5">{inner()}</div>
      </div>
    </div>
  )
}
