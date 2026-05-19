/**
 * Resolves the WebSocket URL for the agent UI.
 *
 * Priority:
 * 1. Hash query `ws`, `wsUrl`, or `relay_ws` — full `ws:` / `wss:` URL (often percent-encoded).
 * 2. Path `/session/:id` — browser UI served by hick-relay (task-37.8); WebSocket is the same path.
 * 3. Default — same origin `/ws` (local agent on :7890, or Vite dev proxy to :7890).
 */
export function resolveWebSocketUrl(): string {
  const hash = window.location.hash.replace(/^#/, '').trim()
  if (hash.includes('=')) {
    const params = new URLSearchParams(hash)
    const explicit =
      params.get('relay_ws') ?? params.get('ws') ?? params.get('wsUrl')
    if (explicit) {
      try {
        return decodeURIComponent(explicit)
      } catch {
        return explicit
      }
    }
  }

  const sessionPath = window.location.pathname.match(/^\/session\/([^/]+)\/?$/)
  if (sessionPath) {
    const wsProto = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    return `${wsProto}//${window.location.host}/session/${sessionPath[1]}`
  }

  const wsProto = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${wsProto}//${window.location.host}/ws`
}
