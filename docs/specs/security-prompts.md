# Security Prompt Specification

## Scope

This document specifies the behaviour of *security prompts*: user-facing
approval requests that gate network requests and tool calls the runtime
security policy has classified as `requires_approval`.

## Definitions

**Security prompt** — any `UserInputRequest::SecurityApproval` emitted by the
network filter or tool-call security layer when a pending action matches a
`requires_approval` rule.

**Approval timeout** — the duration after which, if no user decision has been
recorded, the runtime applies the default verdict.

**Decided-by** — the agent that determined the final verdict: `user` (explicit
response) or `timeout` (no response before expiry).

## Timeout and Default Verdict

### Strict timeout-to-blocked rule

Every security prompt MUST carry a timeout. When that timeout expires without
an explicit user response, the verdict MUST be `Deny`.

This rule is unconditional. There is no configuration that can produce a
timeout-to-allow outcome. The policy field `default_verdict` exists for API
completeness but MUST be ignored at timeout; `Deny` is always applied.

**Rationale**: a timeout-to-allow default would silently authorise actions the
user has not explicitly approved. This is especially hazardous in AFK mode,
where the user is absent by definition. Deny-by-default on timeout is the only
posture consistent with a policy of requiring explicit approval.

### Timeout duration

The timeout is configurable at two levels, evaluated in this order:

1. **Per-rule override**: `timeout_secs` on the specific `requires_approval`
   rule in the agent TOML. Takes precedence when present.
2. **Global default**: `security.approval_timeout_secs` in the agent TOML.
   Applied when no per-rule override exists. Default value: `30`.

The minimum permitted value is `5` seconds. Any value below `5` MUST be
rejected at config-parse time with a descriptive error message.

## Behaviour in AFK Mode

Security prompts behave identically whether or not the agent is in AFK mode.
The same timeout applies; expiry always results in `Deny`.

The AFK modal MUST surface all pending security prompts as interactive cards
with a visible countdown timer so the user can respond before the timeout
expires. If the timeout fires while the user is still absent, the card updates
to show `[TIMED OUT — denied]` and the agent continues with the `Deny` verdict.

See also: the AFK security surfacing specification in TASK-52.6.

## Logging

Every security prompt outcome MUST be written to the XML session log as a
`<security-event>` element:

```xml
<security-event
  kind="network_approval | tool_approval"
  target="&lt;url or tool-name&gt;"
  verdict="allow | deny"
  decided-by="user | timeout"
  requested-at="&lt;ISO-8601&gt;"
  decided-at="&lt;ISO-8601&gt;"
/>
```

Logging MUST occur for every outcome, including timeouts and programmatic
denials that are never shown in a UI.

## UI Requirements

### Approval activation delay

Every security prompt MUST enforce a mandatory 5-second activation delay
before the approve action becomes available. The intent is to ensure the
user reads and understands the prompt before approving, and to make
approval feel meaningfully deliberate rather than a reflexive click.

During the activation delay:

- The approval button or key binding MUST be visually disabled and MUST NOT
  respond to input.
- A secondary countdown (distinct from the approval-timeout countdown)
  MUST be displayed adjacent to the approve control, counting down from 5
  to 0. Labels such as "Allow (5…)" are acceptable.
- Once the activation delay expires, the approve control MUST become active
  and the secondary countdown MUST disappear.
- The deny action MUST be available immediately — the activation delay
  applies only to approval.

The activation delay MUST NOT be configurable. It is a fixed security
property of every security prompt, not a user preference.

**Rationale**: browser extension install prompts use this pattern to
prevent drive-by approvals. An agent security prompt carries at least as
much risk. Making the user wait 5 seconds signals the gravity of the
decision and ensures the prompt content has been read.

### General UI requirements

- The approval-timeout countdown MUST be visible and MUST update at least
  once per second while the prompt is pending.
- A security prompt MUST NOT be dismissable without recording a verdict.
  Closing a modal, navigating away, or terminating the frontend MUST NOT
  count as a verdict — the runtime MUST continue the countdown
  independently of UI state and MUST apply `Deny` on expiry.
- After resolution, the prompt MUST transition in place to a historical
  record showing the verdict and decided-by value. It MUST NOT disappear
  from the conversation log.
- Pending security prompts MUST be surfaced in the conversation timeline
  as interactive nodes (see `docs/specs/user-input.md` § SecurityApproval
  and TASK-54).
