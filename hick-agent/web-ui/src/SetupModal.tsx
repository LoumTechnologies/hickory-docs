import { useState, useCallback, useRef } from 'react'

interface Props {
  onSubmit: (keys: { anthropic?: string; openai?: string; xai?: string }) => void
}

type ProviderKind = 'anthropic' | 'openai' | 'xai'

const ALL_PROVIDERS: ProviderKind[] = ['anthropic', 'openai', 'xai']

const PROVIDER_LABEL: Record<ProviderKind, string> = {
  anthropic: 'Anthropic Claude',
  openai: 'OpenAI',
  xai: 'xAI / Grok',
}

function detectProvider(key: string): ProviderKind | null {
  const k = key.trim()
  if (k.startsWith('sk-ant-')) return 'anthropic'
  if (k.startsWith('xai-'))    return 'xai'
  if (k.startsWith('sk-'))     return 'openai'
  return null
}

function maskKey(key: string): string {
  const visible = Math.max(0, key.length - 4)
  return '•'.repeat(Math.min(visible, 12)) + key.slice(visible)
}

export function SetupModal({ onSubmit }: Props) {
  const [input, setInput] = useState('')
  const [manualProvider, setManualProvider] = useState<ProviderKind>('anthropic')
  const [configured, setConfigured] = useState<Map<ProviderKind, string>>(new Map())
  const [error, setError] = useState<string | null>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  const detected = input.trim() ? detectProvider(input) : null
  const effectiveProvider: ProviderKind | null = input.trim()
    ? (detected ?? manualProvider)
    : null

  const addKey = useCallback(() => {
    const key = input.trim()
    if (!key) return
    const provider = detected ?? manualProvider
    setConfigured((prev) => {
      const next = new Map(prev)
      next.set(provider, key)
      return next
    })
    setInput('')
    setError(null)
    inputRef.current?.focus()
  }, [input, detected, manualProvider])

  const handleSubmit = useCallback(() => {
    // Add current input if non-empty before submitting.
    const key = input.trim()
    let finalConfigured = configured
    if (key) {
      const provider = detected ?? manualProvider
      finalConfigured = new Map(configured)
      finalConfigured.set(provider, key)
    }
    if (finalConfigured.size === 0) {
      setError('Paste at least one API key to continue.')
      return
    }
    onSubmit({
      anthropic: finalConfigured.get('anthropic'),
      openai: finalConfigured.get('openai'),
      xai: finalConfigured.get('xai'),
    })
  }, [input, detected, manualProvider, configured, onSubmit])

  const removeKey = useCallback((provider: ProviderKind) => {
    setConfigured((prev) => {
      const next = new Map(prev)
      next.delete(provider)
      return next
    })
  }, [])

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/90 backdrop-blur-sm">
      <div className="w-full max-w-lg rounded-xl border border-slate-700 bg-slate-900 shadow-2xl">

        {/* Header */}
        <div className="flex items-center rounded-t-xl border-b border-slate-700 bg-slate-800/80 px-6 py-4">
          <span className="text-lg font-bold text-slate-100">Hick Agent — First Run Setup</span>
        </div>

        <div className="space-y-5 px-6 py-5">

          {/* Security callout */}
          <div className="rounded-lg border border-emerald-600/50 bg-emerald-950/40 px-4 py-4 space-y-2">
            <p className="text-sm font-semibold text-emerald-300">
              Your keys never leave your machine.
            </p>
            <p className="text-xs text-emerald-200/80 leading-relaxed">
              Hick's built-in{' '}
              <span className="font-semibold text-emerald-300">secrets scanner</span>{' '}
              detects and redacts credentials before they ever reach the LLM's context
              window. Audit rules in{' '}
              <code className="rounded bg-slate-800 px-1 text-xs text-emerald-200">
                .hick/agent-security.toml
              </code>
              {' '}at any time.
            </p>
          </div>

          {/* Configured providers */}
          {configured.size > 0 && (
            <div className="space-y-1">
              <p className="text-xs font-medium text-slate-400">Configured providers</p>
              {ALL_PROVIDERS.filter((p) => configured.has(p)).map((p) => (
                <div
                  key={p}
                  className="flex items-center justify-between rounded-lg border border-slate-700 bg-slate-800/60 px-3 py-2"
                >
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-semibold text-emerald-400">✓</span>
                    <span className="text-sm font-medium text-slate-200">
                      {PROVIDER_LABEL[p]}
                    </span>
                    <span className="font-mono text-xs text-slate-500">
                      {maskKey(configured.get(p)!)}
                    </span>
                  </div>
                  <button
                    type="button"
                    onClick={() => removeKey(p)}
                    className="ml-3 text-xs text-slate-500 hover:text-red-400"
                    aria-label={`Remove ${PROVIDER_LABEL[p]} key`}
                  >
                    ×
                  </button>
                </div>
              ))}
            </div>
          )}

          {/* Key input */}
          <div className="space-y-2">
            <label className="block text-xs font-medium text-slate-400">
              Paste API key — provider is auto-detected from the prefix
            </label>
            <input
              ref={inputRef}
              type="password"
              autoComplete="off"
              spellCheck={false}
              placeholder="sk-ant-…  /  sk-…  /  xai-…"
              value={input}
              onChange={(e) => { setInput(e.target.value); setError(null) }}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  e.preventDefault()
                  if (input.trim()) addKey()
                  else handleSubmit()
                }
              }}
              className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 font-mono text-sm text-slate-100 placeholder:text-slate-600 focus:border-sky-500 focus:outline-none focus:ring-1 focus:ring-sky-500"
            />

            {/* Detection indicator */}
            {input.trim() && (
              <div className="flex items-center gap-3">
                {detected ? (
                  <p className="text-xs text-emerald-400">
                    Detected: <span className="font-semibold">{PROVIDER_LABEL[detected]}</span>
                  </p>
                ) : (
                  <div className="flex items-center gap-2">
                    <span className="text-xs text-amber-400">Unknown prefix — select provider:</span>
                    <div className="flex gap-1">
                      {ALL_PROVIDERS.map((p) => (
                        <button
                          key={p}
                          type="button"
                          onClick={() => setManualProvider(p)}
                          className={`rounded px-2 py-0.5 text-xs transition-colors ${
                            manualProvider === p
                              ? 'bg-amber-600 text-white font-semibold'
                              : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
                          }`}
                        >
                          {PROVIDER_LABEL[p]}
                        </button>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Error */}
          {error && (
            <p className="text-sm font-medium text-red-400">{error}</p>
          )}

          {/* Footer note */}
          <p className="text-xs text-slate-500">
            Keys are saved to{' '}
            <code className="rounded bg-slate-800 px-1 text-slate-400">~/.hick/keys.toml</code>
            {' '}and loaded automatically on the next launch.
          </p>

          {/* Actions */}
          <div className="flex justify-end gap-3 pt-1">
            {input.trim() && (
              <button
                type="button"
                onClick={addKey}
                className="rounded-lg border border-slate-600 bg-slate-800 px-4 py-2 text-sm font-medium text-slate-200 hover:bg-slate-700 focus:outline-none focus:ring-2 focus:ring-slate-400"
              >
                Add {effectiveProvider ? PROVIDER_LABEL[effectiveProvider] : 'key'}
              </button>
            )}
            <button
              type="button"
              onClick={handleSubmit}
              className="rounded-lg bg-sky-600 px-5 py-2 text-sm font-semibold text-white hover:bg-sky-500 focus:outline-none focus:ring-2 focus:ring-sky-400"
            >
              {configured.size > 0 || input.trim() ? 'Save & Continue' : 'Save & Continue'}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
