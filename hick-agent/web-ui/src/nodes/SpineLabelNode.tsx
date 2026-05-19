interface Props {
  kind: 'info' | 'error'
  text: string
}

export function SpineLabelNode({ kind, text }: Props) {
  const isError = kind === 'error'
  return (
    <div
      className={`mx-auto my-1 flex items-center justify-center gap-1.5 px-3 py-0.5 text-xs ${
        isError ? 'text-red-400' : 'text-slate-400'
      }`}
    >
      <span>{isError ? '✕' : 'ℹ'}</span>
      <span className="max-w-[480px] truncate">{text}</span>
    </div>
  )
}
