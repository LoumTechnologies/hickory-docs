/**
 * Browser-side mirror of the Rust `UiCommand` enum (`hick-agent-sdk` / `ui_event.rs`).
 * Serialized JSON uses `type` in snake_case.
 */
export type UiCommand =
  | { type: 'message'; text: string }
  | { type: 'direct_command'; text: string }
  | { type: 'cancel' }
  | { type: 'quit' }
  | { type: 'expand_node'; node_id: number }
  | { type: 'collapse_node'; node_id: number }
  | { type: 'rewind_to'; node_id: number }

/**
 * Wire format for `UiEvent` — matches `ui_event_wire_value` in `ui_event.rs`.
 * Keep in sync manually with the Rust side.
 */
export type UiEvent =
  | {
      type: 'session_started'
      node_id: number
      model: string | null
      sandbox_image: string | null
      provider_info: string | null
    }
  | {
      type: 'user_turn'
      node_id: number
      parent_id: number | null
      text: string
    }
  | {
      type: 'thinking'
      node_id: number
      parent_id: number | null
      text: string | null
    }
  | {
      type: 'token'
      node_id: number
      text: string
    }
  | {
      type: 'assistant_done'
      node_id: number
      text: string
    }
  | {
      type: 'action_start'
      node_id: number
      parent_id: number | null
      lang: string
      code: string
      collapsed_by_default: boolean
    }
  | {
      type: 'action_end'
      node_id: number
      stdout: string
      stderr: string
      exit_code: number | null
      collapsed_by_default: boolean
    }
  | {
      type: 'sub_agent_spawned'
      node_id: number
      parent_id: number | null
      label: string
    }
  | {
      type: 'sub_agent_done'
      node_id: number
    }
  | {
      type: 'mcp_call'
      node_id: number
      parent_id: number | null
      server: string
      method: string
      collapsed_by_default: boolean
    }
  | {
      type: 'mcp_response'
      node_id: number
      server: string
      method: string
      collapsed_by_default: boolean
    }
  | {
      type: 'security_violation'
      node_id: number
      description: string
    }
  | {
      type: 'network_approval'
      node_id: number
      host: string
      suggestions: string[]
    }
  | {
      type: 'adaptive_card'
      node_id: number
      parent_id: number | null
      json: string
    }
  | {
      type: 'info'
      node_id: number
      text: string
    }
  | {
      type: 'error'
      node_id: number
      text: string
    }
  | {
      type: 'done'
      node_id: number
    }
  | {
      type: 'interrupted'
      node_id: number
    }
  | {
      type: 'branch_started'
      node_id: number
      fork_node_id: number
      label: string
    }
