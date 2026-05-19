interface Props {
  visible: boolean
}

export function BlockingBanner({ visible }: Props) {
  if (!visible) return null
  return (
    <div
      role="alert"
      className="shrink-0 border-b border-amber-500/60 bg-amber-950/80 px-4 py-2 text-center text-sm font-medium text-amber-200"
    >
      ⚠ Waiting for approval — scroll to the prompt to respond.
    </div>
  )
}
