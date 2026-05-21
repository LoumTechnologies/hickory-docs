import { useState, useCallback } from 'react'

interface Props {
  onSubmit: (keys: { anthropic?: string; openai?: string; xai?: string }) => void
}

export function SetupModal({ onSubmit }: Props) {
  const [anthropic, setAnthropic] = useState('')
  const [openai, setOpenai] = useState('')
  const [xai, setXai] = useState('')
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = useCallback(() => {
    const a = anthropic.trim() || undefined
    const o = openai.trim() || undefined
    const x = xai.trim() || undefined
    if (!a && !o && !x) {
      setError('Enter at least one API key to continue.')
      return
    }
    onSubmit({ anthropic: a, openai: o, xai: x })
  }, [anthropic, openai, xai, onSubmit])

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/90 backdrop-blur-sm">
      <div className="w-full max-w-lg rounded-xl border border-slate-700 bg-slate-900 shadow-2xl">

        {/* Header */}
        <div className="flex items-center gap-3 rounded-t-xl border-b border-slate-700 bg-slate-800/80 px-6 py-4">
          <span className="text-lg font-bold text-slate-100">Hick Agent — First Run Setup</span>
        </div>

        <div className="px-6 py-5 space-y-5">

          {/* Security callout */}
          <div className="rounded-lg border border-emerald-600/50 bg-emerald-950/40 px-4 py-4 space-y-2">
            <p className="text-sm font-semibold text-emerald-300">
              Your keys never leave your machine.
            </p>
            <p className="text-xs text-emerald-200/80 leading-relaxed">
              Hick's built-in{' '}
              <span className="font-semibold text-emerald-300">secrets scanner</span>{' '}
              detects and redacts credentials before they ever reach the LLM's context
              window. You can audit the scanner rules in{' '}
              <code className="rounded bg-slate-800 px-1 text-xs text-emerald-200">
                .hick/agent-security.toml
              </code>{' '}
              at any time.
            </p>
          </div>

          {/* Provider fields */}
          <div className="space-y-3">
            <KeyField
              label="Anthropic"
              envVar="ANTHROPIC_API_KEY"
              placeholder="sk-ant-…"
              value={anthropic}
              onChange={(v) => { setAnthropic(v); setError(null) }}
            />
            <KeyField
              label="OpenAI"
              envVar="OPENAI_API_KEY"
              placeholder="sk-…"
              value={openai}
              onChange={(v) => { setOpenai(v); setError(null) }}
            />
            <KeyField
              label="xAI"
              envVar="XAI_API_KEY"
              placeholder="xai-…"
              value={xai}
              onChange={(v) => { setXai(v); setError(null) }}
            />
          </div>

          {/* Error */}
          {error && (
            <p className="text-sm text-red-400 font-medium">{error}</p>
          )}

          {/* Footer note */}
          <p className="text-xs text-slate-500">
            Keys are saved to{' '}
            <code className="rounded bg-slate-800 px-1 text-slate-400">~/.hick/keys.toml</code>{' '}
            and loaded automatically on the next launch. At least one key is required.
          </p>

          {/* Actions */}
          <div className="flex justify-end gap-3 pt-1">
            <button
              type="button"
              onClick={handleSubmit}
              className="rounded-lg bg-sky-600 px-5 py-2 text-sm font-semibold text-white hover:bg-sky-500 focus:outline-none focus:ring-2 focus:ring-sky-400"
            >
              Save &amp; Continue
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}

interface KeyFieldProps {
  label: string
  envVar: string
  placeholder: string
  value: string
  onChange: (v: string) => void
}

function KeyField({ label, envVar, placeholder, value, onChange }: KeyFieldProps) {
  return (
    <div className="space-y-1">
      <label className="flex items-center gap-2 text-xs text-slate-400">
        <span className="font-medium text-slate-300">{label}</span>
        <span className="font-mono text-slate-500">{envVar}</span>
      </label>
      <input
        type="password"
        autoComplete="off"
        spellCheck={false}
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            // Allow bubbling to the parent's submit handler via the button.
          }
        }}
        className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 font-mono text-sm text-slate-100 placeholder:text-slate-600 focus:border-sky-500 focus:outline-none focus:ring-1 focus:ring-sky-500"
      />
    </div>
  )
}
