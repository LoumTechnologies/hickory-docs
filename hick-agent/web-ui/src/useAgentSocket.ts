import { useCallback, useEffect, useRef, useState } from 'react'
import { resolveWebSocketUrl } from './resolveWebSocketUrl'
import type { UiCommand, UiEvent } from './types'

export function useAgentSocket(): {
  events: UiEvent[]
  send: (cmd: UiCommand) => void
  connected: boolean
  /** Browser wall time when the first `session_started` event was received (local). */
  sessionStartedObservedAt: string | null
  /** True when the socket is connected to a server-side replay stream. */
  isServerReplay: boolean
} {
  const [events, setEvents] = useState<UiEvent[]>([])
  const [connected, setConnected] = useState(false)
  const [sessionStartedObservedAt, setSessionStartedObservedAt] = useState<string | null>(null)
  const wsRef = useRef<WebSocket | null>(null)
  const attemptRef = useRef(0)

  // Check once for ?replay= query param.
  const replayPath = new URLSearchParams(window.location.search).get('replay')
  const isServerReplay = replayPath !== null

  const send = useCallback((cmd: UiCommand) => {
    if (isServerReplay) return // read-only in replay mode
    const ws = wsRef.current
    if (!ws || ws.readyState !== WebSocket.OPEN) return
    ws.send(JSON.stringify(cmd))
  }, [isServerReplay])

  useEffect(() => {
    let cancelled = false
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null
    let socket: WebSocket | null = null

    const scheduleReconnect = () => {
      if (cancelled || isServerReplay) return // don't reconnect for replay
      const n = attemptRef.current
      const baseMs = 500 * 2 ** n
      const capped = Math.min(30_000, baseMs)
      attemptRef.current = n + 1
      reconnectTimer = setTimeout(connect, capped)
    }

    const connect = () => {
      if (cancelled) return
      reconnectTimer = null

      const url = isServerReplay
        ? resolveWebSocketUrl('/ws/replay', `path=${encodeURIComponent(replayPath!)}`)
        : resolveWebSocketUrl()
      socket = new WebSocket(url)
      wsRef.current = socket

      socket.onopen = () => {
        attemptRef.current = 0
        if (!cancelled) setConnected(true)
      }

      socket.onclose = () => {
        wsRef.current = null
        if (!cancelled) setConnected(false)
        scheduleReconnect()
      }

      socket.onerror = () => {
        /* Browser fires onclose after error; reconnect there. */
      }

      socket.onmessage = (ev) => {
        if (typeof ev.data !== 'string') return
        try {
          const parsed = JSON.parse(ev.data) as UiEvent
          if (!cancelled) {
            setEvents((prev) => [...prev, parsed])
            if (parsed.type === 'session_started') {
              setSessionStartedObservedAt((prev) => prev ?? new Date().toISOString())
            }
          }
        } catch {
          /* drop malformed frame */
        }
      }
    }

    connect()

    return () => {
      cancelled = true
      if (reconnectTimer !== null) clearTimeout(reconnectTimer)
      attemptRef.current = 0
      socket?.close()
      wsRef.current = null
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  return { events, send, connected, sessionStartedObservedAt, isServerReplay }
}
