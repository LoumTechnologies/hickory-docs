import { useEffect, useRef } from 'react'
import * as AdaptiveCards from 'adaptivecards'

interface Props {
  json: string
}

export function AdaptiveCardNode({ json }: Props) {
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const el = containerRef.current
    if (!el) return

    let card: AdaptiveCards.AdaptiveCard | null = null
    try {
      const parsed = JSON.parse(json)
      card = new AdaptiveCards.AdaptiveCard()
      card.hostConfig = new AdaptiveCards.HostConfig({
        fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace',
        containerStyles: {
          default: { backgroundColor: 'transparent', foregroundColors: { default: { default: '#e2e8f0' } } },
        },
      })
      card.parse(parsed)
      const rendered = card.render()
      if (rendered) {
        el.innerHTML = ''
        el.appendChild(rendered)
      }
    } catch {
      el.textContent = json
    }

    return () => {
      el.innerHTML = ''
    }
  }, [json])

  return (
    <div className="mx-4 my-2 rounded border border-slate-600 bg-slate-800/50 p-3">
      <div className="mb-2 text-xs text-slate-500">Adaptive Card</div>
      <div ref={containerRef} className="adaptive-card-host text-sm text-slate-200" />
    </div>
  )
}
