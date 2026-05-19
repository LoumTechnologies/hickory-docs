/**
 * Client-side parser for Agent Log XML session bundles.
 *
 * Uses the browser's built-in DOMParser — no external XML library needed.
 * Unknown elements are silently skipped so older sessions remain loadable.
 */

import type { UiEvent } from './types'

/** A parsed event with its original recording timestamp (RFC 3339). */
export interface TimestampedEvent {
  event: UiEvent
  /** RFC 3339 timestamp from the `time` attribute on the XML element. */
  timestamp: string
}

let _nextId = 1
function nextId(): number {
  return _nextId++
}

/** Parse a session.xml string and return all mapped events with timestamps. */
export class SessionBundleParser {
  static parse(xmlText: string): TimestampedEvent[] {
    _nextId = 1
    const doc = new DOMParser().parseFromString(xmlText, 'application/xml')
    const parseError = doc.querySelector('parsererror')
    if (parseError) {
      throw new Error(`XML parse error: ${parseError.textContent}`)
    }

    const results: TimestampedEvent[] = []
    // Iterate direct children of <log:session>
    const session = doc.documentElement
    const children = Array.from(session.children)

    // Track pending action node_id so ActionEnd can reuse it.
    let pendingActionId: number | null = null

    for (const el of children) {
      const localName = el.localName
      const time = el.getAttribute('time') ?? new Date(0).toISOString()

      switch (localName) {
        case 'session-start': {
          const node_id = nextId()
          results.push({
            timestamp: time,
            event: {
              type: 'session_started',
              node_id,
              model: el.getAttribute('model'),
              sandbox_image: el.getAttribute('sandbox'),
              provider_info: null,
            },
          })
          break
        }

        case 'user': {
          results.push({
            timestamp: time,
            event: {
              type: 'user_turn',
              node_id: nextId(),
              parent_id: null,
              text: el.textContent ?? '',
            },
          })
          break
        }

        case 'assistant': {
          // Emit Thinking then AssistantDone to match live-session behaviour.
          const thinkingId = nextId()
          const assistantId = nextId()
          results.push({
            timestamp: time,
            event: { type: 'thinking', node_id: thinkingId, parent_id: null, text: null },
          })
          results.push({
            timestamp: time,
            event: {
              type: 'assistant_done',
              node_id: assistantId,
              text: el.textContent ?? '',
            },
          })
          break
        }

        case 'action': {
          const node_id = nextId()
          pendingActionId = node_id
          results.push({
            timestamp: time,
            event: {
              type: 'action_start',
              node_id,
              parent_id: null,
              lang: el.getAttribute('lang') ?? 'sh',
              code: el.textContent ?? '',
              collapsed_by_default: true,
            },
          })
          break
        }

        case 'observation': {
          const node_id = pendingActionId ?? nextId()
          pendingActionId = null
          const exitStr = el.getAttribute('exit')
          const exit_code = exitStr !== null && exitStr !== 'unknown' ? parseInt(exitStr, 10) : null
          results.push({
            timestamp: time,
            event: {
              type: 'action_end',
              node_id,
              stdout: el.textContent ?? '',
              stderr: '',
              exit_code: Number.isNaN(exit_code as number) ? null : exit_code,
              collapsed_by_default: true,
            },
          })
          break
        }

        case 'mcp-call': {
          results.push({
            timestamp: time,
            event: {
              type: 'mcp_call',
              node_id: nextId(),
              parent_id: null,
              server: el.getAttribute('server') ?? 'unknown',
              method: el.getAttribute('method') ?? 'unknown',
              collapsed_by_default: true,
            },
          })
          break
        }

        case 'mcp-response': {
          results.push({
            timestamp: time,
            event: {
              type: 'mcp_response',
              node_id: nextId(),
              server: el.getAttribute('server') ?? 'unknown',
              method: el.getAttribute('method') ?? 'unknown',
              collapsed_by_default: true,
            },
          })
          break
        }

        case 'security-violation': {
          results.push({
            timestamp: time,
            event: {
              type: 'security_violation',
              node_id: nextId(),
              description: el.textContent ?? '',
            },
          })
          break
        }

        case 'session-end': {
          results.push({
            timestamp: time,
            event: { type: 'done', node_id: nextId() },
          })
          break
        }

        // state-transition, diagnostic, and any future elements → skip silently.
        default:
          break
      }
    }

    return results
  }
}
